package com.dragonis.checklist;

import android.content.ContentValues;
import android.content.Context;
import android.graphics.BitmapFactory;
import android.net.Uri;
import android.os.Build;
import android.provider.MediaStore;
import android.widget.ImageView;
import java.io.InputStream;

/* loaded from: classes.dex */
public class ImageUtil {
    public static Uri createNewImageUri(Context context, String str) {
        ContentValues contentValues = new ContentValues();
        contentValues.put("_display_name", str + "_" + System.currentTimeMillis() + ".jpg");
        contentValues.put("mime_type", "image/jpeg");
        if (Build.VERSION.SDK_INT >= 29) {
            contentValues.put("relative_path", "Pictures/ChecklistPhoto");
        }
        return context.getContentResolver().insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues);
    }

    public static void loadThumbnail(Context context, String str, ImageView imageView, int i) {
        if (str == null) {
            imageView.setImageDrawable(null);
            return;
        }
        try {
            Uri parse = Uri.parse(str);
            BitmapFactory.Options options = new BitmapFactory.Options();
            int i2 = 1;
            options.inJustDecodeBounds = true;
            InputStream openInputStream = context.getContentResolver().openInputStream(parse);
            try {
                BitmapFactory.decodeStream(openInputStream, null, options);
                if (openInputStream != null) {
                    openInputStream.close();
                }
                while (true) {
                    int i3 = i * 2;
                    if (options.outWidth / i2 <= i3 || options.outHeight / i2 <= i3) {
                        break;
                    } else {
                        i2 *= 2;
                    }
                }
                BitmapFactory.Options options2 = new BitmapFactory.Options();
                options2.inSampleSize = i2;
                openInputStream = context.getContentResolver().openInputStream(parse);
                try {
                    imageView.setImageBitmap(BitmapFactory.decodeStream(openInputStream, null, options2));
                    if (openInputStream != null) {
                        openInputStream.close();
                    }
                } finally {
                    if (openInputStream != null) {
                        try {
                            openInputStream.close();
                        } catch (Throwable th) {
                            th.addSuppressed(th);
                        }
                    }
                }
            } finally {
            }
        } catch (Exception e) {
            imageView.setImageDrawable(null);
        }
    }
}
