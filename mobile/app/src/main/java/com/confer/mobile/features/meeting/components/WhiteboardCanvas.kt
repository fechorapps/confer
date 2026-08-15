package com.confer.mobile.features.meeting.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.WhiteboardPoint
import com.confer.mobile.core.network.WhiteboardStroke
import com.confer.mobile.core.theme.*
import java.util.UUID
import kotlin.math.abs
import kotlin.math.min

enum class WhiteboardTool {
    PEN,
    HIGHLIGHTER,
    ERASER,
    RECTANGLE,
    CIRCLE,
    LINE
}

private fun parseHexColor(hex: String, defaultColor: Color = PrimaryCyan): Color {
    return try {
        val clean = hex.removePrefix("#")
        when (clean.length) {
            6 -> Color(clean.toLong(16) or 0xFF000000L)
            8 -> Color(clean.toLong(16))
            else -> defaultColor
        }
    } catch (e: Exception) {
        defaultColor
    }
}

@Composable
fun WhiteboardCanvas(
    strokes: List<WhiteboardStroke>,
    myParticipantId: String,
    onDrawStroke: (WhiteboardStroke) -> Unit,
    onUndo: () -> Unit,
    onClear: () -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier
) {
    var currentTool by remember { mutableStateOf(WhiteboardTool.PEN) }
    var selectedColorHex by remember { mutableStateOf("#38BDF8") }
    var selectedStrokeWidth by remember { mutableFloatStateOf(4.0f) }
    var showColorPicker by remember { mutableStateOf(false) }
    var showWidthPicker by remember { mutableStateOf(false) }

    // Active in-progress drawing points
    var currentPoints by remember { mutableStateOf<List<WhiteboardPoint>>(emptyList()) }

    val colors = listOf(
        "#38BDF8", // Cyan
        "#10B981", // Emerald Green
        "#F59E0B", // Amber
        "#E11D48", // Crimson Red
        "#A855F7", // Purple
        "#FFFFFF", // White
        "#0284C7"  // Deep Blue
    )

    val strokeWidths = listOf(
        2.0f to "Thin",
        4.0f to "Medium",
        8.0f to "Thick",
        16.0f to "X-Thick"
    )

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(SurfaceDark)
    ) {
        // Grid Pattern and Stroke Canvas
        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(currentTool, selectedColorHex, selectedStrokeWidth) {
                    detectDragGestures(
                        onDragStart = { offset ->
                            currentPoints = listOf(WhiteboardPoint(offset.x, offset.y))
                        },
                        onDrag = { change, _ ->
                            change.consume()
                            val newPoint = WhiteboardPoint(change.position.x, change.position.y)
                            currentPoints = currentPoints + newPoint
                        },
                        onDragEnd = {
                            if (currentPoints.isNotEmpty()) {
                                val isEraser = currentTool == WhiteboardTool.ERASER
                                val effectiveColor = when (currentTool) {
                                    WhiteboardTool.HIGHLIGHTER -> selectedColorHex
                                    WhiteboardTool.ERASER -> "#121417" // SurfaceDark
                                    else -> selectedColorHex
                                }
                                val effectiveWidth = when (currentTool) {
                                    WhiteboardTool.HIGHLIGHTER -> selectedStrokeWidth * 3f
                                    WhiteboardTool.ERASER -> selectedStrokeWidth * 4f
                                    else -> selectedStrokeWidth
                                }

                                val pointsToSave = when (currentTool) {
                                    WhiteboardTool.PEN, WhiteboardTool.HIGHLIGHTER, WhiteboardTool.ERASER -> currentPoints
                                    WhiteboardTool.LINE, WhiteboardTool.RECTANGLE, WhiteboardTool.CIRCLE -> {
                                        if (currentPoints.size >= 2) {
                                            listOf(currentPoints.first(), currentPoints.last())
                                        } else currentPoints
                                    }
                                }

                                val newStroke = WhiteboardStroke(
                                    id = UUID.randomUUID().toString(),
                                    participantId = myParticipantId,
                                    points = pointsToSave,
                                    color = effectiveColor,
                                    strokeWidth = effectiveWidth,
                                    isEraser = isEraser
                                )
                                onDrawStroke(newStroke)
                                currentPoints = emptyList()
                            }
                        },
                        onDragCancel = {
                            currentPoints = emptyList()
                        }
                    )
                }
        ) {
            // Draw Subtle Dot Grid
            drawDotGrid(dotSpacing = 30f, dotRadius = 1.5f, dotColor = BorderDark.copy(alpha = 0.6f))

            // Draw Completed Strokes
            strokes.forEach { stroke ->
                drawSavedStroke(stroke)
            }

            // Draw Active In-Progress Stroke / Shape
            if (currentPoints.isNotEmpty()) {
                val previewColor = when (currentTool) {
                    WhiteboardTool.HIGHLIGHTER -> parseHexColor(selectedColorHex).copy(alpha = 0.4f)
                    WhiteboardTool.ERASER -> SurfaceDark
                    else -> parseHexColor(selectedColorHex)
                }
                val previewWidth = when (currentTool) {
                    WhiteboardTool.HIGHLIGHTER -> selectedStrokeWidth * 3f
                    WhiteboardTool.ERASER -> selectedStrokeWidth * 4f
                    else -> selectedStrokeWidth
                }

                when (currentTool) {
                    WhiteboardTool.PEN, WhiteboardTool.HIGHLIGHTER, WhiteboardTool.ERASER -> {
                        drawPointsPath(currentPoints, previewColor, previewWidth)
                    }
                    WhiteboardTool.LINE -> {
                        if (currentPoints.size >= 2) {
                            val start = currentPoints.first()
                            val end = currentPoints.last()
                            drawLine(
                                color = previewColor,
                                start = Offset(start.x, start.y),
                                end = Offset(end.x, end.y),
                                strokeWidth = previewWidth,
                                cap = StrokeCap.Round
                            )
                        }
                    }
                    WhiteboardTool.RECTANGLE -> {
                        if (currentPoints.size >= 2) {
                            val p1 = currentPoints.first()
                            val p2 = currentPoints.last()
                            val left = min(p1.x, p2.x)
                            val top = min(p1.y, p2.y)
                            val width = abs(p2.x - p1.x)
                            val height = abs(p2.y - p1.y)
                            drawRect(
                                color = previewColor,
                                topLeft = Offset(left, top),
                                size = Size(width, height),
                                style = Stroke(width = previewWidth)
                            )
                        }
                    }
                    WhiteboardTool.CIRCLE -> {
                        if (currentPoints.size >= 2) {
                            val p1 = currentPoints.first()
                            val p2 = currentPoints.last()
                            val left = min(p1.x, p2.x)
                            val top = min(p1.y, p2.y)
                            val width = abs(p2.x - p1.x)
                            val height = abs(p2.y - p1.y)
                            drawOval(
                                color = previewColor,
                                topLeft = Offset(left, top),
                                size = Size(width, height),
                                style = Stroke(width = previewWidth)
                            )
                        }
                    }
                }
            }
        }

        // Top Navigation & Actions Bar
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp)
                .clip(RoundedCornerShape(16.dp)),
            color = SurfaceDark.copy(alpha = 0.95f),
            tonalElevation = 8.dp,
            border = androidx.compose.foundation.BorderStroke(1.dp, BorderDark)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 12.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        Icons.Outlined.Brush,
                        contentDescription = null,
                        tint = PrimaryCyan,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "Whiteboard",
                        color = Color.White,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "(${strokes.size} strokes)",
                        color = TextMuted,
                        fontSize = 11.sp
                    )
                }

                Row(
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // Undo
                    IconButton(
                        onClick = onUndo,
                        enabled = strokes.isNotEmpty()
                    ) {
                        Icon(
                            Icons.Default.Undo,
                            contentDescription = "Undo",
                            tint = if (strokes.isNotEmpty()) Color.White else TextMuted
                        )
                    }

                    // Clear All
                    IconButton(
                        onClick = onClear,
                        enabled = strokes.isNotEmpty()
                    ) {
                        Icon(
                            Icons.Default.DeleteSweep,
                            contentDescription = "Clear All",
                            tint = if (strokes.isNotEmpty()) AlertRed else TextMuted
                        )
                    }

                    // Close Whiteboard
                    IconButton(onClick = onClose) {
                        Icon(Icons.Default.Close, contentDescription = "Close Whiteboard", tint = Color.White)
                    }
                }
            }
        }

        // Bottom Floating Tool Selector & Color Palette Dock
        Column(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = 16.dp, start = 12.dp, end = 12.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Expanded Color Palette Menu
            AnimatedVisibility(
                visible = showColorPicker,
                enter = fadeIn() + slideInVertically(initialOffsetY = { it / 2 }),
                exit = fadeOut() + slideOutVertically(targetOffsetY = { it / 2 })
            ) {
                Surface(
                    modifier = Modifier
                        .padding(bottom = 8.dp)
                        .clip(RoundedCornerShape(20.dp)),
                    color = SurfaceVariantDark,
                    border = androidx.compose.foundation.BorderStroke(1.dp, BorderDark)
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp),
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        colors.forEach { hex ->
                            val color = parseHexColor(hex)
                            val isSelected = selectedColorHex.equals(hex, ignoreCase = true)
                            Box(
                                modifier = Modifier
                                    .size(28.dp)
                                    .clip(CircleShape)
                                    .background(color)
                                    .border(
                                        width = if (isSelected) 2.dp else 1.dp,
                                        color = if (isSelected) Color.White else Color.Transparent,
                                        shape = CircleShape
                                    )
                                    .clickable {
                                        selectedColorHex = hex
                                        showColorPicker = false
                                    }
                            )
                        }
                    }
                }
            }

            // Expanded Stroke Width Selector
            AnimatedVisibility(
                visible = showWidthPicker,
                enter = fadeIn() + slideInVertically(initialOffsetY = { it / 2 }),
                exit = fadeOut() + slideOutVertically(targetOffsetY = { it / 2 })
            ) {
                Surface(
                    modifier = Modifier
                        .padding(bottom = 8.dp)
                        .clip(RoundedCornerShape(20.dp)),
                    color = SurfaceVariantDark,
                    border = androidx.compose.foundation.BorderStroke(1.dp, BorderDark)
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        strokeWidths.forEach { (width, label) ->
                            val isSelected = selectedStrokeWidth == width
                            FilterChip(
                                selected = isSelected,
                                onClick = {
                                    selectedStrokeWidth = width
                                    showWidthPicker = false
                                },
                                label = { Text(label, fontSize = 11.sp) },
                                colors = FilterChipDefaults.filterChipColors(
                                    selectedContainerColor = PrimaryCyan,
                                    selectedLabelColor = BackgroundDark,
                                    containerColor = SurfaceDark,
                                    labelColor = TextPrimary
                                )
                            )
                        }
                    }
                }
            }

            // Main Dock Row
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(24.dp)),
                color = SurfaceDark.copy(alpha = 0.95f),
                tonalElevation = 8.dp,
                border = androidx.compose.foundation.BorderStroke(1.dp, BorderDark)
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 8.dp, vertical = 6.dp)
                        .horizontalScroll(rememberScrollState()),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // Pen Tool
                    ToolIconButton(
                        isSelected = currentTool == WhiteboardTool.PEN,
                        onClick = { currentTool = WhiteboardTool.PEN },
                        icon = Icons.Default.Edit,
                        contentDescription = "Pen"
                    )

                    // Highlighter Tool
                    ToolIconButton(
                        isSelected = currentTool == WhiteboardTool.HIGHLIGHTER,
                        onClick = { currentTool = WhiteboardTool.HIGHLIGHTER },
                        icon = Icons.Default.Highlight,
                        contentDescription = "Highlighter"
                    )

                    // Eraser Tool
                    ToolIconButton(
                        isSelected = currentTool == WhiteboardTool.ERASER,
                        onClick = { currentTool = WhiteboardTool.ERASER },
                        icon = Icons.Default.AutoFixNormal,
                        contentDescription = "Eraser"
                    )

                    // Line Tool
                    ToolIconButton(
                        isSelected = currentTool == WhiteboardTool.LINE,
                        onClick = { currentTool = WhiteboardTool.LINE },
                        icon = Icons.Default.HorizontalRule,
                        contentDescription = "Line"
                    )

                    // Rectangle Tool
                    ToolIconButton(
                        isSelected = currentTool == WhiteboardTool.RECTANGLE,
                        onClick = { currentTool = WhiteboardTool.RECTANGLE },
                        icon = Icons.Default.CropSquare,
                        contentDescription = "Rectangle"
                    )

                    // Circle Tool
                    ToolIconButton(
                        isSelected = currentTool == WhiteboardTool.CIRCLE,
                        onClick = { currentTool = WhiteboardTool.CIRCLE },
                        icon = Icons.Default.PanoramaFishEye,
                        contentDescription = "Circle"
                    )

                    VerticalDivider(modifier = Modifier.height(24.dp), color = BorderDark)

                    // Color Picker Button
                    IconButton(
                        onClick = {
                            showColorPicker = !showColorPicker
                            showWidthPicker = false
                        },
                        modifier = Modifier.size(38.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(24.dp)
                                .clip(CircleShape)
                                .background(parseHexColor(selectedColorHex))
                                .border(1.5.dp, Color.White, CircleShape)
                        )
                    }

                    // Stroke Width Button
                    IconButton(
                        onClick = {
                            showWidthPicker = !showWidthPicker
                            showColorPicker = false
                        },
                        colors = IconButtonDefaults.iconButtonColors(
                            containerColor = if (showWidthPicker) PrimaryCyan else SurfaceVariantDark
                        ),
                        modifier = Modifier.size(38.dp)
                    ) {
                        Icon(
                            Icons.Default.LineWeight,
                            contentDescription = "Stroke Width",
                            tint = if (showWidthPicker) BackgroundDark else Color.White,
                            modifier = Modifier.size(18.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ToolIconButton(
    isSelected: Boolean,
    onClick: () -> Unit,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    contentDescription: String
) {
    IconButton(
        onClick = onClick,
        colors = IconButtonDefaults.iconButtonColors(
            containerColor = if (isSelected) PrimaryCyan else SurfaceVariantDark
        ),
        modifier = Modifier.size(38.dp)
    ) {
        Icon(
            icon,
            contentDescription = contentDescription,
            tint = if (isSelected) BackgroundDark else Color.White,
            modifier = Modifier.size(18.dp)
        )
    }
}

private fun DrawScope.drawDotGrid(dotSpacing: Float, dotRadius: Float, dotColor: Color) {
    val width = size.width
    val height = size.height

    var x = dotSpacing / 2
    while (x < width) {
        var y = dotSpacing / 2
        while (y < height) {
            drawCircle(
                color = dotColor,
                radius = dotRadius,
                center = Offset(x, y)
            )
            y += dotSpacing
        }
        x += dotSpacing
    }
}

private fun DrawScope.drawSavedStroke(stroke: WhiteboardStroke) {
    val pts = stroke.points
    if (pts.isEmpty()) return

    val color = parseHexColor(stroke.color)
    val width = stroke.strokeWidth

    if (pts.size == 2 && (stroke.color.startsWith("#") || stroke.isEraser)) {
        // If it's a simple 2-point stroke, render line
        drawLine(
            color = color,
            start = Offset(pts[0].x, pts[0].y),
            end = Offset(pts[1].x, pts[1].y),
            strokeWidth = width,
            cap = StrokeCap.Round
        )
    } else {
        drawPointsPath(pts, color, width)
    }
}

private fun DrawScope.drawPointsPath(points: List<WhiteboardPoint>, color: Color, width: Float) {
    if (points.isEmpty()) return
    if (points.size == 1) {
        drawCircle(
            color = color,
            radius = width / 2,
            center = Offset(points[0].x, points[0].y)
        )
        return
    }

    val path = Path()
    path.moveTo(points[0].x, points[0].y)

    for (i in 1 until points.size) {
        val p0 = points[i - 1]
        val p1 = points[i]
        val midX = (p0.x + p1.x) / 2
        val midY = (p0.y + p1.y) / 2
        path.quadraticTo(p0.x, p0.y, midX, midY)
    }
    path.lineTo(points.last().x, points.last().y)

    drawPath(
        path = path,
        color = color,
        style = Stroke(
            width = width,
            cap = StrokeCap.Round,
            join = StrokeJoin.Round
        )
    )
}
