package com.dragonis.checklist;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.DialogInterface;
import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.text.TextUtils;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.EditText;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/* loaded from: classes.dex */
public class AddEditItemActivity extends Activity {
    private static final int REQ_CAMERA_ICON = 10;
    private static final int REQ_CAMERA_PERM = 20;
    private static final int REQ_GALLERY_ICON = 11;
    private TextView btnDelete;
    private LinearLayout containerChecklist;
    private EditText editName;
    private ImageView imgPreview;
    private boolean isNew;
    private ChecklistItem item;
    private Uri pendingCameraUri;
    private final List<View> rowViews = new ArrayList();
    private Store store;

    @Override // android.app.Activity
    protected void onCreate(Bundle bundle) {
        super.onCreate(bundle);
        setContentView(R.layout.activity_add_edit);
        this.store = Store.get(this);
        String stringExtra = getIntent().getStringExtra("item_id");
        if (stringExtra != null) {
            this.item = this.store.findById(stringExtra);
        }
        this.isNew = this.item == null;
        if (this.item == null) {
            this.item = new ChecklistItem();
        }
        this.imgPreview = (ImageView) findViewById(R.id.img_icon_preview);
        this.editName = (EditText) findViewById(R.id.edit_name);
        this.containerChecklist = (LinearLayout) findViewById(R.id.container_checklist);
        this.btnDelete = (TextView) findViewById(R.id.btn_delete);
        ((TextView) findViewById(R.id.txt_title)).setText(this.isNew ? "Nouvel élément" : "Modifier l'élément");
        findViewById(R.id.btn_back).setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda3
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                AddEditItemActivity.this.lambda$onCreate$0(view);
            }
        });
        findViewById(R.id.btn_pick_icon).setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda4
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                AddEditItemActivity.this.lambda$onCreate$1(view);
            }
        });
        findViewById(R.id.btn_add_row).setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda5
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                AddEditItemActivity.this.lambda$onCreate$2(view);
            }
        });
        findViewById(R.id.btn_save).setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda6
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                AddEditItemActivity.this.lambda$onCreate$3(view);
            }
        });
        this.editName.setText(this.item.name);
        if (this.item.iconUri != null) {
            ImageUtil.loadThumbnail(this, this.item.iconUri, this.imgPreview, 400);
        }
        if (this.item.checklist != null) {
            Iterator<String> it = this.item.checklist.iterator();
            while (it.hasNext()) {
                addChecklistRow(it.next());
            }
        }
        if (this.rowViews.isEmpty()) {
            addChecklistRow("");
        }
        if (!this.isNew) {
            this.btnDelete.setVisibility(0);
            this.btnDelete.setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda7
                @Override // android.view.View.OnClickListener
                public final void onClick(View view) {
                    AddEditItemActivity.this.lambda$onCreate$4(view);
                }
            });
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$0(View view) {
        finish();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$1(View view) {
        showIconPickerDialog();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$2(View view) {
        addChecklistRow("");
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$3(View view) {
        save();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$4(View view) {
        confirmDelete();
    }

    private void showIconPickerDialog() {
        new AlertDialog.Builder(this).setItems(new CharSequence[]{"Prendre une photo", "Choisir depuis la galerie"}, new DialogInterface.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda0
            @Override // android.content.DialogInterface.OnClickListener
            public final void onClick(DialogInterface dialogInterface, int i) {
                AddEditItemActivity.this.lambda$showIconPickerDialog$5(dialogInterface, i);
            }
        }).show();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$showIconPickerDialog$5(DialogInterface dialogInterface, int i) {
        if (i != 0) {
            launchGallery();
        } else {
            launchCamera();
        }
    }

    private void launchCamera() {
        if (Build.VERSION.SDK_INT >= 23 && checkSelfPermission("android.permission.CAMERA") != 0) {
            requestPermissions(new String[]{"android.permission.CAMERA"}, REQ_CAMERA_PERM);
            return;
        }
        this.pendingCameraUri = ImageUtil.createNewImageUri(this, "icon");
        if (this.pendingCameraUri == null) {
            Toast.makeText(this, "Impossible de créer le fichier photo", 0).show();
            return;
        }
        Intent intent = new Intent("android.media.action.IMAGE_CAPTURE");
        intent.putExtra("output", this.pendingCameraUri);
        startActivityForResult(intent, REQ_CAMERA_ICON);
    }

    private void launchGallery() {
        Intent intent = new Intent("android.intent.action.OPEN_DOCUMENT");
        intent.addCategory("android.intent.category.OPENABLE");
        intent.setType("image/*");
        intent.addFlags(1);
        startActivityForResult(intent, REQ_GALLERY_ICON);
    }

    @Override // android.app.Activity
    public void onRequestPermissionsResult(int i, String[] strArr, int[] iArr) {
        super.onRequestPermissionsResult(i, strArr, iArr);
        if (i == REQ_CAMERA_PERM && iArr.length > 0 && iArr[0] == 0) {
            launchCamera();
        }
    }

    @Override // android.app.Activity
    protected void onActivityResult(int i, int i2, Intent intent) {
        super.onActivityResult(i, i2, intent);
        if (i2 != -1) {
            return;
        }
        if (i == REQ_CAMERA_ICON && this.pendingCameraUri != null) {
            this.item.iconUri = this.pendingCameraUri.toString();
            ImageUtil.loadThumbnail(this, this.item.iconUri, this.imgPreview, 400);
        } else if (i == REQ_GALLERY_ICON && intent != null && intent.getData() != null) {
            Uri data = intent.getData();
            try {
                getContentResolver().takePersistableUriPermission(data, 1);
            } catch (SecurityException e) {
            }
            this.item.iconUri = data.toString();
            ImageUtil.loadThumbnail(this, this.item.iconUri, this.imgPreview, 400);
        }
    }

    private void addChecklistRow(String str) {
        final View inflate = LayoutInflater.from(this).inflate(R.layout.row_checklist_edit, (ViewGroup) this.containerChecklist, false);
        ((EditText) inflate.findViewById(R.id.edit_row_text)).setText(str);
        inflate.findViewById(R.id.btn_row_delete).setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda2
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                AddEditItemActivity.this.lambda$addChecklistRow$6(inflate, view);
            }
        });
        this.containerChecklist.addView(inflate);
        this.rowViews.add(inflate);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$addChecklistRow$6(View view, View view2) {
        this.containerChecklist.removeView(view);
        this.rowViews.remove(view);
    }

    private void save() {
        String trim = this.editName.getText().toString().trim();
        if (TextUtils.isEmpty(trim)) {
            Toast.makeText(this, "Donne un nom à l'élément", 0).show();
            return;
        }
        ArrayList arrayList = new ArrayList();
        Iterator<View> it = this.rowViews.iterator();
        while (it.hasNext()) {
            String trim2 = ((EditText) it.next().findViewById(R.id.edit_row_text)).getText().toString().trim();
            if (!TextUtils.isEmpty(trim2)) {
                arrayList.add(trim2);
            }
        }
        if (arrayList.isEmpty()) {
            Toast.makeText(this, "Ajoute au moins une étape à la check-list", 0).show();
            return;
        }
        this.item.name = trim;
        this.item.checklist = arrayList;
        if (this.item.progress != null && this.item.progress.checked.size() != arrayList.size()) {
            this.item.progress = null;
        }
        if (this.isNew) {
            this.store.addItem(this.item);
        } else {
            this.store.save();
        }
        finish();
    }

    private void confirmDelete() {
        new AlertDialog.Builder(this).setTitle("Supprimer « " + this.item.name + " » ?").setMessage("La check-list et la progression du jour seront supprimées. L'historique des photos passées reste sur l'appareil.").setPositiveButton("Supprimer", new DialogInterface.OnClickListener() { // from class: com.dragonis.checklist.AddEditItemActivity$$ExternalSyntheticLambda1
            @Override // android.content.DialogInterface.OnClickListener
            public final void onClick(DialogInterface dialogInterface, int i) {
                AddEditItemActivity.this.lambda$confirmDelete$7(dialogInterface, i);
            }
        }).setNegativeButton("Annuler", (DialogInterface.OnClickListener) null).show();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$confirmDelete$7(DialogInterface dialogInterface, int i) {
        this.store.removeItem(this.item.id);
        finish();
    }
}
