package com.dragonis.checklist;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.util.TypedValue;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.CheckBox;
import android.widget.CompoundButton;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/* loaded from: classes.dex */
public class SessionActivity extends Activity {
    private static final int REQ_CAMERA_PERM = 41;
    private static final int REQ_PHOTO = 40;
    private TextView btnChecklistDone;
    private TextView btnPhotosDone;
    private ImageView btnTakePhoto;
    private LinearLayout containerChecklistCheck;
    private LinearLayout containerPhotos;
    private ChecklistItem item;
    private LinearLayout panelChecklist;
    private LinearLayout panelPhotos;
    private Uri pendingPhotoUri;
    private Progress progress;
    private Store store;
    private TextView txtPhotoCounter;
    private TextView txtStep;
    private TextView txtTitle;

    @Override // android.app.Activity
    protected void onCreate(Bundle bundle) {
        super.onCreate(bundle);
        setContentView(R.layout.activity_session);
        this.store = Store.get(this);
        this.item = this.store.findById(getIntent().getStringExtra("item_id"));
        if (this.item == null) {
            finish();
            return;
        }
        this.txtTitle = (TextView) findViewById(R.id.txt_title);
        this.txtStep = (TextView) findViewById(R.id.txt_step);
        this.panelPhotos = (LinearLayout) findViewById(R.id.panel_photos);
        this.panelChecklist = (LinearLayout) findViewById(R.id.panel_checklist);
        this.txtPhotoCounter = (TextView) findViewById(R.id.txt_photo_counter);
        this.containerPhotos = (LinearLayout) findViewById(R.id.container_photos);
        this.btnTakePhoto = (ImageView) findViewById(R.id.btn_take_photo);
        this.btnPhotosDone = (TextView) findViewById(R.id.btn_photos_done);
        this.containerChecklistCheck = (LinearLayout) findViewById(R.id.container_checklist_check);
        this.btnChecklistDone = (TextView) findViewById(R.id.btn_checklist_done);
        this.txtTitle.setText(this.item.name);
        findViewById(R.id.btn_back).setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.SessionActivity$$ExternalSyntheticLambda1
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                SessionActivity.this.lambda$onCreate$0(view);
            }
        });
        this.btnTakePhoto.setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.SessionActivity$$ExternalSyntheticLambda2
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                SessionActivity.this.lambda$onCreate$1(view);
            }
        });
        this.btnPhotosDone.setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.SessionActivity$$ExternalSyntheticLambda3
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                SessionActivity.this.lambda$onCreate$2(view);
            }
        });
        this.btnChecklistDone.setOnClickListener(new View.OnClickListener() { // from class: com.dragonis.checklist.SessionActivity$$ExternalSyntheticLambda4
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                SessionActivity.this.lambda$onCreate$3(view);
            }
        });
        initProgress();
        renderPhase();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$0(View view) {
        finish();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$1(View view) {
        takePhoto();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$2(View view) {
        onPhotosDoneClicked();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$3(View view) {
        onChecklistDoneClicked();
    }

    @Override // android.app.Activity
    protected void onResume() {
        super.onResume();
        this.item.purgeStaleProgress();
        initProgress();
        renderPhase();
    }

    @Override // android.app.Activity
    protected void onPause() {
        super.onPause();
        persist();
    }

    private void initProgress() {
        if (this.item.progress == null) {
            Progress progress = new Progress();
            progress.date = DateUtil.today();
            progress.phase = Progress.PHASE_BEFORE;
            for (int i = 0; i < this.item.checklist.size(); i++) {
                progress.checked.add(false);
            }
            this.item.progress = progress;
        }
        this.progress = this.item.progress;
    }

    private void persist() {
        this.store.save();
    }

    private int dp(int i) {
        return (int) TypedValue.applyDimension(1, i, getResources().getDisplayMetrics());
    }

    /* JADX WARN: Can't fix incorrect switch cases order, some code will duplicate */
    private void renderPhase() {
        char c;
        this.panelPhotos.setVisibility(8);
        this.panelChecklist.setVisibility(8);
        String str = this.progress.phase;
        switch (str.hashCode()) {
            case -1109073530:
                if (str.equals(Progress.PHASE_CHECKLIST)) {
                    c = 1;
                    break;
                }
                c = 65535;
                break;
            case 62197180:
                if (str.equals(Progress.PHASE_AFTER)) {
                    c = 2;
                    break;
                }
                c = 65535;
                break;
            case 1955410815:
                if (str.equals(Progress.PHASE_BEFORE)) {
                    c = 0;
                    break;
                }
                c = 65535;
                break;
            default:
                c = 65535;
                break;
        }
        switch (c) {
            case 0:
                this.txtStep.setText("Étape 1/3 · Photos avant");
                this.panelPhotos.setVisibility(0);
                this.btnPhotosDone.setVisibility(0);
                renderPhotoPanel(this.progress.beforePhotos, -1);
                return;
            case 1:
                this.txtStep.setText("Étape 2/3 · Check-list");
                this.panelChecklist.setVisibility(0);
                renderChecklist();
                return;
            case 2:
                this.txtStep.setText("Étape 3/3 · Photos après");
                this.panelPhotos.setVisibility(0);
                this.btnPhotosDone.setVisibility(8);
                renderPhotoPanel(this.progress.afterPhotos, this.progress.targetCount);
                return;
            default:
                return;
        }
    }

    private void renderPhotoPanel(List<String> list, int i) {
        this.containerPhotos.removeAllViews();
        int dp = dp(90);
        int dp2 = dp(8);
        for (String str : list) {
            ImageView imageView = new ImageView(this);
            LinearLayout.LayoutParams layoutParams = new LinearLayout.LayoutParams(dp, dp);
            layoutParams.setMargins(dp2, dp2, dp2, dp2);
            imageView.setLayoutParams(layoutParams);
            imageView.setScaleType(ImageView.ScaleType.CENTER_CROP);
            imageView.setBackgroundResource(R.drawable.bg_thumb);
            ImageUtil.loadThumbnail(this, str, imageView, dp);
            this.containerPhotos.addView(imageView);
        }
        if (i < 0) {
            this.txtPhotoCounter.setText(list.size() + " photo(s) prise(s)");
            this.btnPhotosDone.setText("Terminer les photos (" + list.size() + ")");
            this.btnPhotosDone.setAlpha(list.isEmpty() ? 0.5f : 1.0f);
            return;
        }
        this.txtPhotoCounter.setText(list.size() + " / " + i + " photos");
    }

    private void renderChecklist() {
        this.containerChecklistCheck.removeAllViews();
        final int i = 0;
        while (i < this.item.checklist.size()) {
            View inflate = LayoutInflater.from(this).inflate(R.layout.row_checklist_check, (ViewGroup) this.containerChecklistCheck, false);
            TextView textView = (TextView) inflate.findViewById(R.id.txt_row_label);
            CheckBox checkBox = (CheckBox) inflate.findViewById(R.id.check_row);
            textView.setText(this.item.checklist.get(i));
            checkBox.setChecked(i < this.progress.checked.size() && this.progress.checked.get(i).booleanValue());
            checkBox.setOnCheckedChangeListener(new CompoundButton.OnCheckedChangeListener() { // from class: com.dragonis.checklist.SessionActivity$$ExternalSyntheticLambda0
                @Override // android.widget.CompoundButton.OnCheckedChangeListener
                public final void onCheckedChanged(CompoundButton compoundButton, boolean z) {
                    SessionActivity.this.lambda$renderChecklist$4(i, compoundButton, z);
                }
            });
            this.containerChecklistCheck.addView(inflate);
            i++;
        }
        updateChecklistDoneButtonState();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$renderChecklist$4(int i, CompoundButton compoundButton, boolean z) {
        while (this.progress.checked.size() <= i) {
            this.progress.checked.add(false);
        }
        this.progress.checked.set(i, Boolean.valueOf(z));
        persist();
        updateChecklistDoneButtonState();
    }

    private boolean allChecked() {
        if (this.progress.checked.size() != this.item.checklist.size()) {
            return false;
        }
        Iterator<Boolean> it = this.progress.checked.iterator();
        while (it.hasNext()) {
            if (!it.next().booleanValue()) {
                return false;
            }
        }
        return true;
    }

    private void updateChecklistDoneButtonState() {
        this.btnChecklistDone.setAlpha(allChecked() ? 1.0f : 0.5f);
    }

    private void onPhotosDoneClicked() {
        if (this.progress.beforePhotos.isEmpty()) {
            Toast.makeText(this, "Prends au moins une photo avant de continuer", 0).show();
            return;
        }
        this.progress.targetCount = this.progress.beforePhotos.size();
        this.progress.phase = Progress.PHASE_CHECKLIST;
        persist();
        renderPhase();
    }

    private void onChecklistDoneClicked() {
        if (!allChecked()) {
            Toast.makeText(this, "Coche toutes les étapes de la check-list", 0).show();
            return;
        }
        this.progress.phase = Progress.PHASE_AFTER;
        persist();
        renderPhase();
    }

    private void takePhoto() {
        if (Build.VERSION.SDK_INT >= 23 && checkSelfPermission("android.permission.CAMERA") != 0) {
            requestPermissions(new String[]{"android.permission.CAMERA"}, REQ_CAMERA_PERM);
            return;
        }
        if (Progress.PHASE_AFTER.equals(this.progress.phase) && this.progress.afterPhotos.size() >= this.progress.targetCount) {
            return;
        }
        this.pendingPhotoUri = ImageUtil.createNewImageUri(this, Progress.PHASE_AFTER.equals(this.progress.phase) ? "after" : "before");
        if (this.pendingPhotoUri == null) {
            Toast.makeText(this, "Impossible de créer le fichier photo", 0).show();
            return;
        }
        Intent intent = new Intent("android.media.action.IMAGE_CAPTURE");
        intent.putExtra("output", this.pendingPhotoUri);
        startActivityForResult(intent, REQ_PHOTO);
    }

    @Override // android.app.Activity
    public void onRequestPermissionsResult(int i, String[] strArr, int[] iArr) {
        super.onRequestPermissionsResult(i, strArr, iArr);
        if (i == REQ_CAMERA_PERM && iArr.length > 0 && iArr[0] == 0) {
            takePhoto();
        }
    }

    @Override // android.app.Activity
    protected void onActivityResult(int i, int i2, Intent intent) {
        super.onActivityResult(i, i2, intent);
        if (i != REQ_PHOTO || i2 != -1 || this.pendingPhotoUri == null) {
            return;
        }
        String uri = this.pendingPhotoUri.toString();
        if (Progress.PHASE_BEFORE.equals(this.progress.phase)) {
            this.progress.beforePhotos.add(uri);
        } else if (Progress.PHASE_AFTER.equals(this.progress.phase)) {
            this.progress.afterPhotos.add(uri);
        }
        persist();
        if (Progress.PHASE_AFTER.equals(this.progress.phase) && this.progress.afterPhotos.size() >= this.progress.targetCount) {
            finishSession();
        } else {
            renderPhase();
        }
    }

    private void finishSession() {
        Session session = new Session();
        session.date = this.progress.date;
        session.beforePhotos = this.progress.beforePhotos;
        session.afterPhotos = this.progress.afterPhotos;
        session.checklistSnapshot = new ArrayList(this.item.checklist);
        this.item.history.add(session);
        this.item.progress = null;
        persist();
        Toast.makeText(this, "Session terminée ✅", 1).show();
        finish();
    }
}
