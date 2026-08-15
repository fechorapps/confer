package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.sp

@Composable
fun WatermarkOverlay(
    displayName: String,
    modifier: Modifier = Modifier
) {
    if (displayName.isBlank()) return

    val textMeasurer = rememberTextMeasurer()
    val watermarkText = "$displayName • Confer"
    val watermarkColor = Color.White.copy(alpha = 0.12f)
    val textStyle = TextStyle(
        color = watermarkColor,
        fontSize = 13.sp,
        fontWeight = FontWeight.Medium
    )

    Canvas(modifier = modifier.fillMaxSize()) {
        val stepX = 220f
        val stepY = 140f

        val cols = (size.width / stepX).toInt() + 2
        val rows = (size.height / stepY).toInt() + 2

        for (r in -1..rows) {
            val offsetX = if (r % 2 == 0) 0f else stepX / 2f
            for (c in -1..cols) {
                val x = c * stepX + offsetX
                val y = r * stepY

                rotate(degrees = -25f, pivot = Offset(x, y)) {
                    drawText(
                        textMeasurer = textMeasurer,
                        text = watermarkText,
                        topLeft = Offset(x, y),
                        style = textStyle
                    )
                }
            }
        }
    }
}
