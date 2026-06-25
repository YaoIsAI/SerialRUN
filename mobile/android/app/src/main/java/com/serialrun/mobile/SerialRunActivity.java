package com.serialrun.mobile;

import android.app.Activity;
import android.content.Intent;
import android.hardware.usb.UsbDevice;
import android.hardware.usb.UsbManager;
import android.os.Bundle;
import android.util.Log;

/**
 * SerialRUN Mobile - Main Activity
 */
public class SerialRunActivity extends Activity {
    private static final String TAG = "SerialRUN";

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Log.i(TAG, "SerialRUN Mobile started");
    }

    @Override
    protected void onResume() {
        super.onResume();
        UsbManager usbManager = (UsbManager) getSystemService(USB_SERVICE);
        if (usbManager != null) {
            Log.i(TAG, "USB devices: " + usbManager.getDeviceList().size());
        }
    }
}
