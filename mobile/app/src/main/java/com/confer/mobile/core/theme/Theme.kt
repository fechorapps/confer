package com.confer.mobile.core.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable

private val DarkColorScheme = darkColorScheme(
    primary = PrimaryCyan,
    secondary = SecondaryBlue,
    tertiary = AccentGreen,
    background = BackgroundDark,
    surface = SurfaceDark,
    surfaceVariant = SurfaceVariantDark,
    error = AlertRed,
    onPrimary = BackgroundDark,
    onBackground = TextPrimary,
    onSurface = TextPrimary,
    outline = BorderDark
)

@Composable
fun ConferTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        typography = Typography,
        content = content
    )
}
