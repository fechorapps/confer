/* Minimal libvpx VP8 encode/decode wrapper for Confer desktop client.
 *
 * This file is compiled as a static library by build.rs and linked against
 * the system libvpx. It exposes a tiny C API so the Rust side does not need
 * to generate libvpx bindings.
 */

#include <stdlib.h>
#include <string.h>

#include <vpx/vpx_encoder.h>
#include <vpx/vpx_decoder.h>
#include <vpx/vp8cx.h>
#include <vpx/vp8dx.h>
#include <vpx/vpx_image.h>

/* Opaque encoder handle. */
typedef struct {
    vpx_codec_ctx_t codec;
    vpx_image_t img;
    unsigned int deadline;
} vpx_enc_t;

/* Opaque decoder handle. */
typedef struct {
    vpx_codec_ctx_t codec;
} vpx_dec_t;

/* Encode result returned to Rust. */
typedef struct {
    int ok;
    const uint8_t *data;
    size_t len;
    int keyframe;
    int has_more;
} vpx_enc_packet_t;

/* Decode result returned to Rust. */
typedef struct {
    int ok;
    int has_frame;
    unsigned int width;
    unsigned int height;
    unsigned int stride_y;
    unsigned int stride_u;
    unsigned int stride_v;
    const uint8_t *plane_y;
    const uint8_t *plane_u;
    const uint8_t *plane_v;
} vpx_dec_frame_t;

vpx_enc_t *vpx_enc_create(unsigned int width, unsigned int height, unsigned int bitrate_kbps,
                          unsigned int fps_num, unsigned int fps_den, unsigned int deadline_us) {
    vpx_enc_t *enc = calloc(1, sizeof(vpx_enc_t));
    if (!enc) return NULL;

    vpx_codec_enc_cfg_t cfg;
    vpx_codec_err_t res = vpx_codec_enc_config_default(vpx_codec_vp8_cx(), &cfg, 0);
    if (res != VPX_CODEC_OK) {
        free(enc);
        return NULL;
    }

    cfg.g_w = width;
    cfg.g_h = height;
    cfg.rc_target_bitrate = bitrate_kbps;
    cfg.g_timebase.num = fps_den;
    cfg.g_timebase.den = fps_num;
    /* 1 lagged frame is fine for video calls; 0 would reduce latency further. */
    cfg.g_lag_in_frames = 0;
    /* Disable multithreaded lookahead to keep latency low. */
    cfg.g_threads = 1;
    /* Allow dynamic keyframe distance. */
    cfg.kf_mode = VPX_KF_AUTO;
    cfg.kf_min_dist = 0;
    cfg.kf_max_dist = fps_num * 2; /* ~2 seconds */

    res = vpx_codec_enc_init_ver(&enc->codec, vpx_codec_vp8_cx(), &cfg,
                                 0, VPX_ENCODER_ABI_VERSION);
    if (res != VPX_CODEC_OK) {
        free(enc);
        return NULL;
    }

    /* CPU usage 5 is a good realtime balance. Lower = faster/lower quality. */
    vpx_codec_control(&enc->codec, VP8E_SET_CPUUSED, 5);
    vpx_codec_control(&enc->codec, VP8E_SET_STATIC_THRESHOLD, 0);
    vpx_codec_control(&enc->codec, VP8E_SET_ENABLEAUTOALTREF, 0);

    if (!vpx_img_alloc(&enc->img, VPX_IMG_FMT_I420, width, height, 1)) {
        vpx_codec_destroy(&enc->codec);
        free(enc);
        return NULL;
    }

    enc->deadline = deadline_us;
    return enc;
}

void vpx_enc_destroy(vpx_enc_t *enc) {
    if (!enc) return;
    vpx_codec_destroy(&enc->codec);
    vpx_img_free(&enc->img);
    free(enc);
}

const char *vpx_enc_error(vpx_enc_t *enc) {
    if (!enc) return "null encoder";
    return vpx_codec_error(&enc->codec);
}

/* Copy an I420 frame into the internal image. Planes must be tightly packed or
 * follow the supplied strides. */
int vpx_enc_encode(vpx_enc_t *enc, const uint8_t *y, unsigned int stride_y,
                   const uint8_t *u, unsigned int stride_u,
                   const uint8_t *v, unsigned int stride_v,
                   vpx_codec_pts_t pts, unsigned long duration,
                   vpx_codec_flags_t flags) {
    if (!enc || !y || !u || !v) return -1;

    vpx_image_t *img = &enc->img;

    /* Copy each plane respecting strides. */
    for (unsigned int row = 0; row < img->h; ++row) {
        memcpy(img->planes[VPX_PLANE_Y] + row * img->stride[VPX_PLANE_Y],
               y + row * stride_y, img->w);
    }
    for (unsigned int row = 0; row < img->h / 2; ++row) {
        memcpy(img->planes[VPX_PLANE_U] + row * img->stride[VPX_PLANE_U],
               u + row * stride_u, img->w / 2);
        memcpy(img->planes[VPX_PLANE_V] + row * img->stride[VPX_PLANE_V],
               v + row * stride_v, img->w / 2);
    }

    vpx_codec_err_t res = vpx_codec_encode(&enc->codec, img, pts, duration, flags, enc->deadline);
    return res == VPX_CODEC_OK ? 0 : -1;
}

/* Get the next encoded packet. Call repeatedly while has_more == 1.
 * The returned data pointer is owned by libvpx and is only valid until the
 * next call into the encoder. */
vpx_enc_packet_t vpx_enc_get_packet(vpx_enc_t *enc) {
    vpx_enc_packet_t out = {0};
    if (!enc) return out;

    vpx_codec_iter_t iter = NULL;
    const vpx_codec_cx_pkt_t *pkt = vpx_codec_get_cx_data(&enc->codec, &iter);
    if (!pkt) return out;

    out.ok = 1;
    if (pkt->kind == VPX_CODEC_CX_FRAME_PKT) {
        out.data = (const uint8_t *)pkt->data.frame.buf;
        out.len = pkt->data.frame.sz;
        out.keyframe = (pkt->data.frame.flags & VPX_FRAME_IS_KEY) != 0;
    } else {
        out.ok = 0;
    }

    /* Peek to tell Rust whether another packet is available. */
    const vpx_codec_cx_pkt_t *next = vpx_codec_get_cx_data(&enc->codec, &iter);
    out.has_more = next != NULL;
    return out;
}

/* Flush any pending encoder packets. Call with no input frame. */
int vpx_enc_flush(vpx_enc_t *enc, vpx_codec_pts_t pts) {
    if (!enc) return -1;
    vpx_codec_err_t res = vpx_codec_encode(&enc->codec, NULL, pts, 1, 0, enc->deadline);
    return res == VPX_CODEC_OK ? 0 : -1;
}

vpx_dec_t *vpx_dec_create(void) {
    vpx_dec_t *dec = calloc(1, sizeof(vpx_dec_t));
    if (!dec) return NULL;

    vpx_codec_dec_cfg_t cfg = {0};
    cfg.threads = 1;
    vpx_codec_err_t res = vpx_codec_dec_init_ver(&dec->codec, vpx_codec_vp8_dx(), &cfg,
                                                  0, VPX_DECODER_ABI_VERSION);
    if (res != VPX_CODEC_OK) {
        free(dec);
        return NULL;
    }
    return dec;
}

void vpx_dec_destroy(vpx_dec_t *dec) {
    if (!dec) return;
    vpx_codec_destroy(&dec->codec);
    free(dec);
}

const char *vpx_dec_error(vpx_dec_t *dec) {
    if (!dec) return "null decoder";
    return vpx_codec_error(&dec->codec);
}

int vpx_dec_decode(vpx_dec_t *dec, const uint8_t *data, size_t len) {
    if (!dec || !data) return -1;
    vpx_codec_err_t res = vpx_codec_decode(&dec->codec, data, (unsigned int)len, NULL, 0);
    return res == VPX_CODEC_OK ? 0 : -1;
}

/* Get the next decoded frame. The plane pointers are owned by libvpx and are
 * only valid until the next decode call. */
vpx_dec_frame_t vpx_dec_get_frame(vpx_dec_t *dec) {
    vpx_dec_frame_t out = {0};
    if (!dec) return out;

    vpx_codec_iter_t iter = NULL;
    vpx_image_t *img = vpx_codec_get_frame(&dec->codec, &iter);
    if (!img) return out;

    out.ok = 1;
    out.has_frame = 1;
    out.width = img->d_w;
    out.height = img->d_h;
    out.stride_y = img->stride[VPX_PLANE_Y];
    out.stride_u = img->stride[VPX_PLANE_U];
    out.stride_v = img->stride[VPX_PLANE_V];
    out.plane_y = img->planes[VPX_PLANE_Y];
    out.plane_u = img->planes[VPX_PLANE_U];
    out.plane_v = img->planes[VPX_PLANE_V];
    return out;
}
