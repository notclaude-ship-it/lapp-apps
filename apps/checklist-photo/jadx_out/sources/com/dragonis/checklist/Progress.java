package com.dragonis.checklist;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

/* loaded from: classes.dex */
public class Progress {
    public static final String PHASE_AFTER = "AFTER";
    public static final String PHASE_BEFORE = "BEFORE";
    public static final String PHASE_CHECKLIST = "CHECKLIST";
    public String date;
    public String phase = PHASE_BEFORE;
    public List<String> beforePhotos = new ArrayList();
    public List<String> afterPhotos = new ArrayList();
    public List<Boolean> checked = new ArrayList();
    public int targetCount = 0;

    public JSONObject toJson() throws JSONException {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("date", this.date);
        jSONObject.put("phase", this.phase);
        jSONObject.put("beforePhotos", new JSONArray((Collection) this.beforePhotos));
        jSONObject.put("afterPhotos", new JSONArray((Collection) this.afterPhotos));
        JSONArray jSONArray = new JSONArray();
        Iterator<Boolean> it = this.checked.iterator();
        while (it.hasNext()) {
            jSONArray.put(it.next());
        }
        jSONObject.put("checked", jSONArray);
        jSONObject.put("targetCount", this.targetCount);
        return jSONObject;
    }

    public static Progress fromJson(JSONObject jSONObject) throws JSONException {
        Progress progress = new Progress();
        progress.date = jSONObject.optString("date", "");
        progress.phase = jSONObject.optString("phase", PHASE_BEFORE);
        progress.beforePhotos = jsonToList(jSONObject.optJSONArray("beforePhotos"));
        progress.afterPhotos = jsonToList(jSONObject.optJSONArray("afterPhotos"));
        progress.targetCount = jSONObject.optInt("targetCount", 0);
        progress.checked = new ArrayList();
        JSONArray optJSONArray = jSONObject.optJSONArray("checked");
        if (optJSONArray != null) {
            for (int i = 0; i < optJSONArray.length(); i++) {
                progress.checked.add(Boolean.valueOf(optJSONArray.optBoolean(i, false)));
            }
        }
        return progress;
    }

    private static List<String> jsonToList(JSONArray jSONArray) {
        ArrayList arrayList = new ArrayList();
        if (jSONArray == null) {
            return arrayList;
        }
        for (int i = 0; i < jSONArray.length(); i++) {
            arrayList.add(jSONArray.optString(i));
        }
        return arrayList;
    }
}
