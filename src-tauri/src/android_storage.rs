//! Android shared-storage helpers (Documents folder via MANAGE_EXTERNAL_STORAGE).

#![cfg(target_os = "android")]

use jni::objects::{JObject, JValue};
use jni::JavaVM;

/// True when the OS has granted all-files access (Android 11+) or legacy write (older).
pub fn has_all_files_access() -> bool {
    match try_has_all_files_access() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("android storage check failed: {error}");
            false
        }
    }
}

/// Opens the system screen where the user can grant “All files access” for Tonic.
pub fn request_all_files_access() -> Result<(), String> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let context = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

    let sdk = env
        .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
        .map_err(|e| e.to_string())?
        .i()
        .map_err(|e| e.to_string())?;

    if sdk >= 30 {
        let action = env
            .new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION")
            .map_err(|e| e.to_string())?;
        let intent = env
            .new_object(
                "android/content/Intent",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&action)],
            )
            .map_err(|e| e.to_string())?;

        let pkg = env
            .call_method(&context, "getPackageName", "()Ljava/lang/String;", &[])
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let prefix = env.new_string("package:").map_err(|e| e.to_string())?;
        let uri_str = env
            .call_method(
                &prefix,
                "concat",
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(&pkg)],
            )
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&uri_str)],
            )
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let intent = env
            .call_method(
                &intent,
                "setData",
                "(Landroid/net/Uri;)Landroid/content/Intent;",
                &[JValue::Object(&uri)],
            )
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;

        let flag = env
            .get_static_field("android/content/Intent", "FLAG_ACTIVITY_NEW_TASK", "I")
            .map_err(|e| e.to_string())?
            .i()
            .map_err(|e| e.to_string())?;
        let _ = env
            .call_method(
                &intent,
                "addFlags",
                "(I)Landroid/content/Intent;",
                &[JValue::Int(flag)],
            )
            .map_err(|e| e.to_string())?;

        env.call_method(
            &context,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        // Legacy: open app settings so the user can enable Storage.
        let action = env
            .new_string("android.settings.APPLICATION_DETAILS_SETTINGS")
            .map_err(|e| e.to_string())?;
        let intent = env
            .new_object(
                "android/content/Intent",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&action)],
            )
            .map_err(|e| e.to_string())?;
        let pkg = env
            .call_method(&context, "getPackageName", "()Ljava/lang/String;", &[])
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let prefix = env.new_string("package:").map_err(|e| e.to_string())?;
        let uri_str = env
            .call_method(
                &prefix,
                "concat",
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(&pkg)],
            )
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&uri_str)],
            )
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let intent = env
            .call_method(
                &intent,
                "setData",
                "(Landroid/net/Uri;)Landroid/content/Intent;",
                &[JValue::Object(&uri)],
            )
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let flag = env
            .get_static_field("android/content/Intent", "FLAG_ACTIVITY_NEW_TASK", "I")
            .map_err(|e| e.to_string())?
            .i()
            .map_err(|e| e.to_string())?;
        let _ = env
            .call_method(
                &intent,
                "addFlags",
                "(I)Landroid/content/Intent;",
                &[JValue::Int(flag)],
            )
            .map_err(|e| e.to_string())?;
        env.call_method(
            &context,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn try_has_all_files_access() -> Result<bool, String> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;

    let sdk = env
        .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
        .map_err(|e| e.to_string())?
        .i()
        .map_err(|e| e.to_string())?;

    if sdk >= 30 {
        let granted = env
            .call_static_method(
                "android/os/Environment",
                "isExternalStorageManager",
                "()Z",
                &[],
            )
            .map_err(|e| e.to_string())?
            .z()
            .map_err(|e| e.to_string())?;
        Ok(granted)
    } else {
        let context = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };
        let permission = env
            .new_string("android.permission.WRITE_EXTERNAL_STORAGE")
            .map_err(|e| e.to_string())?;
        let result = env
            .call_method(
                &context,
                "checkSelfPermission",
                "(Ljava/lang/String;)I",
                &[JValue::Object(&permission)],
            )
            .map_err(|e| e.to_string())?
            .i()
            .map_err(|e| e.to_string())?;
        // PackageManager.PERMISSION_GRANTED == 0
        Ok(result == 0)
    }
}
