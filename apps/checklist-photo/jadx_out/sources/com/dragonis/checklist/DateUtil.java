package com.dragonis.checklist;

import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.Locale;

/* loaded from: classes.dex */
public class DateUtil {
    public static String today() {
        return new SimpleDateFormat("yyyy-MM-dd", Locale.US).format(new Date());
    }

    public static String prettyToday() {
        return new SimpleDateFormat("d MMMM yyyy", Locale.FRANCE).format(new Date());
    }
}
