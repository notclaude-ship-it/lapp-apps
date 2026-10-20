package com.dragonis.checklist;

import android.content.Context;
import android.util.Log;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

/* loaded from: classes.dex */
public class Store {
    private static final String FILE_NAME = "data.json";
    private static final String TAG = "Store";
    private static Store instance;
    private final File file;
    private final List<ChecklistItem> items = new ArrayList();

    private Store(Context context) {
        this.file = new File(context.getApplicationContext().getFilesDir(), FILE_NAME);
        load();
    }

    public static synchronized Store get(Context context) {
        Store store;
        synchronized (Store.class) {
            if (instance == null) {
                instance = new Store(context);
            }
            store = instance;
        }
        return store;
    }

    public List<ChecklistItem> getItems() {
        return this.items;
    }

    public ChecklistItem findById(String str) {
        for (ChecklistItem checklistItem : this.items) {
            if (checklistItem.id.equals(str)) {
                return checklistItem;
            }
        }
        return null;
    }

    public void addItem(ChecklistItem checklistItem) {
        this.items.add(checklistItem);
        save();
    }

    public void removeItem(String str) {
        ChecklistItem findById = findById(str);
        if (findById != null) {
            this.items.remove(findById);
        }
        save();
    }

    public synchronized void save() {
        JSONObject jSONObject;
        File file;
        FileOutputStream fileOutputStream;
        OutputStreamWriter outputStreamWriter;
        try {
            JSONArray jSONArray = new JSONArray();
            Iterator<ChecklistItem> it = this.items.iterator();
            while (it.hasNext()) {
                jSONArray.put(it.next().toJson());
            }
            jSONObject = new JSONObject();
            jSONObject.put("items", jSONArray);
            file = new File(this.file.getParentFile(), "data.json.tmp");
            fileOutputStream = new FileOutputStream(file);
            try {
                outputStreamWriter = new OutputStreamWriter(fileOutputStream, StandardCharsets.UTF_8);
            } catch (Throwable th) {
                try {
                    fileOutputStream.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (Exception e) {
            Log.e(TAG, "Erreur de sauvegarde", e);
        }
        try {
            outputStreamWriter.write(jSONObject.toString());
            outputStreamWriter.close();
            fileOutputStream.close();
            file.renameTo(this.file);
        } finally {
        }
    }

    private synchronized void load() {
        int i;
        this.items.clear();
        if (this.file.exists()) {
            try {
                StringBuilder sb = new StringBuilder();
                FileInputStream fileInputStream = new FileInputStream(this.file);
                try {
                    InputStreamReader inputStreamReader = new InputStreamReader(fileInputStream, StandardCharsets.UTF_8);
                    try {
                        char[] cArr = new char[4096];
                        while (true) {
                            int read = inputStreamReader.read(cArr);
                            if (read == -1) {
                                break;
                            } else {
                                sb.append(cArr, 0, read);
                            }
                        }
                        inputStreamReader.close();
                        fileInputStream.close();
                        JSONArray optJSONArray = new JSONObject(sb.toString()).optJSONArray("items");
                        if (optJSONArray != null) {
                            for (i = 0; i < optJSONArray.length(); i++) {
                                this.items.add(ChecklistItem.fromJson(optJSONArray.getJSONObject(i)));
                            }
                        }
                    } finally {
                    }
                } catch (Throwable th) {
                    try {
                        fileInputStream.close();
                    } catch (Throwable th2) {
                        th.addSuppressed(th2);
                    }
                    throw th;
                }
            } catch (IOException | JSONException e) {
                Log.e(TAG, "Erreur de chargement", e);
            }
        }
    }
}
