package com.dragonis.checklist;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

/* loaded from: classes.dex */
public class ChecklistItem {
    public String iconUri;
    public String name;
    public Progress progress;
    public List<String> checklist = new ArrayList();
    public List<Session> history = new ArrayList();
    public String id = UUID.randomUUID().toString();

    public boolean isConfigured() {
        return (this.checklist == null || this.checklist.isEmpty()) ? false : true;
    }

    public void purgeStaleProgress() {
        if (this.progress != null && !DateUtil.today().equals(this.progress.date)) {
            this.progress = null;
        }
    }

    public JSONObject toJson() throws JSONException {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("id", this.id);
        jSONObject.put("name", this.name);
        if (this.iconUri != null) {
            jSONObject.put("iconUri", this.iconUri);
        }
        jSONObject.put("checklist", new JSONArray((Collection) this.checklist));
        if (this.progress != null) {
            jSONObject.put("progress", this.progress.toJson());
        }
        JSONArray jSONArray = new JSONArray();
        Iterator<Session> it = this.history.iterator();
        while (it.hasNext()) {
            jSONArray.put(it.next().toJson());
        }
        jSONObject.put("history", jSONArray);
        return jSONObject;
    }

    public static ChecklistItem fromJson(JSONObject jSONObject) throws JSONException {
        ChecklistItem checklistItem = new ChecklistItem();
        checklistItem.id = jSONObject.optString("id", UUID.randomUUID().toString());
        checklistItem.name = jSONObject.optString("name", "");
        checklistItem.iconUri = jSONObject.isNull("iconUri") ? null : jSONObject.optString("iconUri", null);
        checklistItem.checklist = new ArrayList();
        JSONArray optJSONArray = jSONObject.optJSONArray("checklist");
        if (optJSONArray != null) {
            for (int i = 0; i < optJSONArray.length(); i++) {
                checklistItem.checklist.add(optJSONArray.optString(i));
            }
        }
        JSONObject optJSONObject = jSONObject.optJSONObject("progress");
        checklistItem.progress = optJSONObject != null ? Progress.fromJson(optJSONObject) : null;
        checklistItem.history = new ArrayList();
        JSONArray optJSONArray2 = jSONObject.optJSONArray("history");
        if (optJSONArray2 != null) {
            for (int i2 = 0; i2 < optJSONArray2.length(); i2++) {
                checklistItem.history.add(Session.fromJson(optJSONArray2.getJSONObject(i2)));
            }
        }
        return checklistItem;
    }
}
