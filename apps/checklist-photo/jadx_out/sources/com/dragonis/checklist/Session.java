package com.dragonis.checklist;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

/* loaded from: classes.dex */
public class Session {
    public String date;
    public List<String> beforePhotos = new ArrayList();
    public List<String> afterPhotos = new ArrayList();
    public List<String> checklistSnapshot = new ArrayList();

    public JSONObject toJson() throws JSONException {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("date", this.date);
        jSONObject.put("beforePhotos", new JSONArray((Collection) this.beforePhotos));
        jSONObject.put("afterPhotos", new JSONArray((Collection) this.afterPhotos));
        jSONObject.put("checklistSnapshot", new JSONArray((Collection) this.checklistSnapshot));
        return jSONObject;
    }

    public static Session fromJson(JSONObject jSONObject) throws JSONException {
        Session session = new Session();
        session.date = jSONObject.optString("date", "");
        session.beforePhotos = toList(jSONObject.optJSONArray("beforePhotos"));
        session.afterPhotos = toList(jSONObject.optJSONArray("afterPhotos"));
        session.checklistSnapshot = toList(jSONObject.optJSONArray("checklistSnapshot"));
        return session;
    }

    private static List<String> toList(JSONArray jSONArray) {
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
