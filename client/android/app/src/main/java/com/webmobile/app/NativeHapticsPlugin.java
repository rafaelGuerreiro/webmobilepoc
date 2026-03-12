package com.webmobile.app;

import android.app.Activity;
import android.view.HapticFeedbackConstants;
import android.view.View;
import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;

@CapacitorPlugin(name = "NativeHaptics")
public class NativeHapticsPlugin extends Plugin {

    @PluginMethod
    public void light(PluginCall call) {
        performFeedback(call, HapticFeedbackConstants.KEYBOARD_TAP);
    }

    @PluginMethod
    public void hard(PluginCall call) {
        performFeedback(call, HapticFeedbackConstants.LONG_PRESS);
    }

    private void performFeedback(PluginCall call, int feedbackConstant) {
        Activity activity = getActivity();
        if (activity == null) {
            call.reject("Native haptics are unavailable because the activity is missing.");
            return;
        }

        activity.runOnUiThread(() -> {
            View decorView = activity.getWindow() != null ? activity.getWindow().getDecorView() : null;
            if (decorView == null) {
                call.reject("Native haptics are unavailable because the window view is missing.");
                return;
            }

            decorView.performHapticFeedback(feedbackConstant);
            call.resolve();
        });
    }
}
