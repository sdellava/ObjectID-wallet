package com.impierce.identity_wallet

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.webkit.WebView

class MainActivity : TauriActivity() {
    private val mainHandler = Handler(Looper.getMainLooper())
    private var objectIdWebView: WebView? = null
    private var wakeReceiverRegistered = false
    @Volatile private var frontendReady = false
    @Volatile private var bootWatchdogToken = 0

    private val wakeReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            val action = intent?.action ?: return
            if (action == Intent.ACTION_SCREEN_ON || action == Intent.ACTION_USER_PRESENT) {
                frontendReady = false
                scheduleShellLoadIfBlank("wake:$action", 500)
                scheduleShellLoadIfBlank("wake:$action", 2000)
                scheduleShellLoadIfBlank("wake:$action", 5000)
                scheduleFrontendReadyCheck("wake:$action", 1500)
                scheduleFrontendReadyCheck("wake:$action", 4000)
                scheduleFrontendReadyCheck("wake:$action", 7000)
                scheduleBootWatchdog("wake:$action", 14000)
            }
        }
    }

    companion object {
        init {
            System.loadLibrary("objectid")
        }
    }

    private external fun java_init(context: android.content.Context)

    override fun onCreate(savedInstanceState: Bundle?) {
        Log.i("ObjectID", "MainActivity onCreate")
        super.onCreate(savedInstanceState)
        Log.i("ObjectID", "MainActivity onCreate: initializing Android context")
        java_init(this)
        registerWakeReceiver()
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        objectIdWebView = webView
        frontendReady = false
        Log.i("ObjectID", "WebView created (url=${webView.url}, progress=${webView.progress})")
        scheduleShellLoadIfBlank("webview-create", 1500)
        scheduleShellLoadIfBlank("webview-create", 3500)
        scheduleShellLoadIfBlank("webview-create", 7000)
        scheduleFrontendReadyCheck("webview-create", 6000)
        scheduleFrontendReadyCheck("webview-create", 11000)
        scheduleBootWatchdog("webview-create", 14000)
    }

    override fun onResume() {
        super.onResume()
        frontendReady = false
        scheduleShellLoadIfBlank("resume", 1500)
        scheduleShellLoadIfBlank("resume", 3500)
        scheduleShellLoadIfBlank("resume", 7000)
        scheduleFrontendReadyCheck("resume", 6000)
        scheduleFrontendReadyCheck("resume", 11000)
        scheduleBootWatchdog("resume", 14000)
    }

    override fun onPause() {
        bootWatchdogToken += 1
        super.onPause()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) {
            scheduleShellLoadIfBlank("window-focus", 2500)
            scheduleFrontendReadyCheck("window-focus", 5000)
        }
    }

    override fun onDestroy() {
        bootWatchdogToken += 1
        unregisterWakeReceiver()
        super.onDestroy()
    }

    private fun scheduleShellLoadIfBlank(reason: String, delayMs: Long) {
        mainHandler.postDelayed({
            loadShellIfBlank(reason)
        }, delayMs)
    }

    private fun scheduleFrontendReadyCheck(reason: String, delayMs: Long) {
        mainHandler.postDelayed({
            ensureFrontendReady(reason)
        }, delayMs)
    }

    private fun loadShellIfBlank(reason: String) {
        val webView = objectIdWebView
        if (webView == null) {
            Log.w("ObjectID", "WebView is not available on $reason")
            return
        }

        val currentUrl = webView.url

        val shellUrl = currentUrl == "http://tauri.localhost" || currentUrl == "http://tauri.localhost/"
        val shellStalled = shellUrl && webView.progress < 100

        if (currentUrl.isNullOrBlank() || currentUrl == "about:blank" || shellStalled) {
            Log.w(
                "ObjectID",
                "WebView shell is not ready on $reason (url=$currentUrl, progress=${webView.progress}), loading ObjectID shell"
            )
            webView.stopLoading()
            webView.loadUrl("http://tauri.localhost")
        }
    }

    private fun ensureFrontendReady(reason: String) {
        val webView = objectIdWebView
        if (webView == null) {
            Log.w("ObjectID", "WebView is not available for frontend ready check on $reason")
            return
        }

        var answered = false
        webView.evaluateJavascript(
            "(function(){return document.documentElement.dataset.objectidReady === 'true' ? 'true' : (location.href || 'blank');})()"
        ) { result ->
            answered = true
            if (result == "\"true\"") {
                frontendReady = true
                Log.i("ObjectID", "Frontend is ready on $reason")
                return@evaluateJavascript
            }

            reloadObjectIdShell(webView, "Frontend is not ready on $reason (result=$result)")
        }

        mainHandler.postDelayed({
            if (!answered) {
                reloadObjectIdShell(webView, "Frontend ready check timed out on $reason")
            }
        }, 750)
    }

    private fun reloadObjectIdShell(webView: WebView, reason: String) {
        Log.w(
            "ObjectID",
            "$reason (url=${webView.url}, progress=${webView.progress}), reloading ObjectID shell"
        )
        webView.stopLoading()
        webView.loadUrl("http://tauri.localhost")
    }

    private fun scheduleBootWatchdog(reason: String, delayMs: Long) {
        val token = bootWatchdogToken + 1
        bootWatchdogToken = token

        Thread {
            try {
                Thread.sleep(delayMs)
            } catch (_: InterruptedException) {
                return@Thread
            }

            if (frontendReady || token != bootWatchdogToken || isFinishing || isDestroyed) {
                return@Thread
            }

            Log.e("ObjectID", "Frontend boot stalled on $reason, relaunching app process")
            relaunchAppProcess()
        }.apply {
            name = "ObjectIDBootWatchdog"
            isDaemon = true
            start()
        }
    }

    private fun relaunchAppProcess() {
        val launchIntent = packageManager.getLaunchIntentForPackage(packageName)
            ?: Intent(this, MainActivity::class.java)

        launchIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)

        try {
            applicationContext.startActivity(launchIntent)
            Thread.sleep(750)
        } catch (error: Exception) {
            Log.e("ObjectID", "Failed to relaunch app before process exit", error)
        }

        android.os.Process.killProcess(android.os.Process.myPid())
        Runtime.getRuntime().exit(0)
    }

    private fun registerWakeReceiver() {
        if (wakeReceiverRegistered) return

        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_SCREEN_ON)
            addAction(Intent.ACTION_USER_PRESENT)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            registerReceiver(wakeReceiver, filter, Context.RECEIVER_NOT_EXPORTED)
        } else {
            @Suppress("DEPRECATION")
            registerReceiver(wakeReceiver, filter)
        }

        wakeReceiverRegistered = true
    }

    private fun unregisterWakeReceiver() {
        if (!wakeReceiverRegistered) return

        unregisterReceiver(wakeReceiver)
        wakeReceiverRegistered = false
    }
}
