package com.teatrogestion.app

import android.Manifest
import android.content.pm.PackageManager
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.Settings
import android.view.WindowManager
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.content.FileProvider
import java.io.File
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

class MainActivity : TauriActivity() {
    private val prefs by lazy { getSharedPreferences("tg_ui", MODE_PRIVATE) }

    /** Pantalla completa: la barra de arriba del teléfono se esconde y reaparece un momento al deslizar desde el borde. */
    private var fullscreen = true

    private fun applyFullscreen() {
        val controller = WindowCompat.getInsetsController(window, window.decorView)
        controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        if (fullscreen) controller.hide(WindowInsetsCompat.Type.statusBars())
        else controller.show(WindowInsetsCompat.Type.statusBars())
        // Con la barra escondida la app usa también la franja de la cámara frontal.
        window.attributes = window.attributes.apply {
            layoutInDisplayCutoutMode =
                if (fullscreen) WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
                else WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_DEFAULT
        }
        window.decorView.findViewById<android.view.View>(android.R.id.content)?.let { ViewCompat.requestApplyInsets(it) }
    }

    /** Interruptor «Pantalla completa» del menú de la app; la elección queda guardada. */
    inner class ScreenMode {
        @JavascriptInterface
        fun isFullscreen(): Boolean = fullscreen

        @JavascriptInterface
        fun setFullscreen(on: Boolean) {
            prefs.edit().putBoolean("fullscreen", on).apply()
            runOnUiThread {
                fullscreen = on
                applyFullscreen()
            }
        }
    }

    /** Entrega al instalador del sistema un APK ya descargado y verificado en la carpeta de caché de la app. */
    inner class UpdateInstaller {
        @JavascriptInterface
        fun installApk(path: String): String {
            val file = File(path)
            val cache = cacheDir.canonicalFile
            if (!file.canonicalFile.startsWith(cache) || !file.isFile) return "archivo-invalido"
            if (!packageManager.canRequestPackageInstalls()) {
                // Primera vez: el sistema pide permitir instalar desde esta app.
                startActivity(
                    Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:$packageName"))
                        .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                )
                return "permiso-pendiente"
            }
            val uri = FileProvider.getUriForFile(this@MainActivity, "$packageName.fileprovider", file)
            startActivity(
                Intent(Intent.ACTION_VIEW)
                    .setDataAndType(uri, "application/vnd.android.package-archive")
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
            )
            return "ok"
        }
    }

    override fun onWebViewCreate(webView: WebView) {
        webView.addJavascriptInterface(UpdateInstaller(), "TGUpdate")
        webView.addJavascriptInterface(ScreenMode(), "TGScreen")
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        fullscreen = prefs.getBoolean("fullscreen", true)
        // La app reparte sola los bordes (barras y cámara frontal) con el relleno de abajo.
        WindowCompat.setDecorFitsSystemWindows(window, false)
        val root = window.decorView.findViewById<android.view.View>(android.R.id.content)
        ViewCompat.setOnApplyWindowInsetsListener(root) { view, insets ->
            val types = if (fullscreen) WindowInsetsCompat.Type.systemBars()
                else WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            val bars = insets.getInsets(types)
            view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
            insets
        }
        applyFullscreen()

        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.CAMERA), 1001)
        }
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        // Al volver de otra app o de un diálogo, Android puede mostrar la barra otra vez.
        if (hasFocus && fullscreen) applyFullscreen()
    }
}
