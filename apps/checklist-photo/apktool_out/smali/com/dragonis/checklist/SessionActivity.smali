.class public Lcom/dragonis/checklist/SessionActivity;
.super Landroid/app/Activity;
.source "SessionActivity.java"


# static fields
.field private static final REQ_CAMERA_PERM:I = 0x29

.field private static final REQ_PHOTO:I = 0x28


# instance fields
.field private btnChecklistDone:Landroid/widget/TextView;

.field private btnPhotosDone:Landroid/widget/TextView;

.field private btnTakePhoto:Landroid/widget/ImageView;

.field private containerChecklistCheck:Landroid/widget/LinearLayout;

.field private containerPhotos:Landroid/widget/LinearLayout;

.field private item:Lcom/dragonis/checklist/ChecklistItem;

.field private panelChecklist:Landroid/widget/LinearLayout;

.field private panelPhotos:Landroid/widget/LinearLayout;

.field private pendingPhotoUri:Landroid/net/Uri;

.field private progress:Lcom/dragonis/checklist/Progress;

.field private store:Lcom/dragonis/checklist/Store;

.field private txtPhotoCounter:Landroid/widget/TextView;

.field private txtStep:Landroid/widget/TextView;

.field private txtTitle:Landroid/widget/TextView;


# direct methods
.method public static synthetic $r8$lambda$SuvTC_bOnhVs8BEYU_-uQagicL8(Lcom/dragonis/checklist/SessionActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->lambda$onCreate$3(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$VEqoMaqdhpJTtEJmmSQkQOTD9dM(Lcom/dragonis/checklist/SessionActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->lambda$onCreate$2(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$ekbclqpnaxGy08H1DXYzjLP-Cuw(Lcom/dragonis/checklist/SessionActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->lambda$onCreate$1(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$fR__geWDmb2rAGosHv007B1u_a4(Lcom/dragonis/checklist/SessionActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->lambda$onCreate$0(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$sWRNK0H5Ef2UTnGHK-AGR8OBvVw(Lcom/dragonis/checklist/SessionActivity;ILandroid/widget/CompoundButton;Z)V
    .locals 0

    invoke-direct {p0, p1, p2, p3}, Lcom/dragonis/checklist/SessionActivity;->lambda$renderChecklist$4(ILandroid/widget/CompoundButton;Z)V

    return-void
.end method

.method public constructor <init>()V
    .locals 0

    .line 20
    invoke-direct {p0}, Landroid/app/Activity;-><init>()V

    return-void
.end method

.method private allChecked()Z
    .locals 3

    .line 176
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-interface {v0}, Ljava/util/List;->size()I

    move-result v0

    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v1, v1, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-interface {v1}, Ljava/util/List;->size()I

    move-result v1

    const/4 v2, 0x0

    if-eq v0, v1, :cond_0

    return v2

    .line 177
    :cond_0
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-interface {v0}, Ljava/util/List;->iterator()Ljava/util/Iterator;

    move-result-object v0

    :cond_1
    invoke-interface {v0}, Ljava/util/Iterator;->hasNext()Z

    move-result v1

    if-eqz v1, :cond_2

    invoke-interface {v0}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v1

    check-cast v1, Ljava/lang/Boolean;

    invoke-virtual {v1}, Ljava/lang/Boolean;->booleanValue()Z

    move-result v1

    if-nez v1, :cond_1

    return v2

    .line 178
    :cond_2
    const/4 v0, 0x1

    return v0
.end method

.method private dp(I)I
    .locals 2

    .line 100
    int-to-float p1, p1

    invoke-virtual {p0}, Lcom/dragonis/checklist/SessionActivity;->getResources()Landroid/content/res/Resources;

    move-result-object v0

    invoke-virtual {v0}, Landroid/content/res/Resources;->getDisplayMetrics()Landroid/util/DisplayMetrics;

    move-result-object v0

    const/4 v1, 0x1

    invoke-static {v1, p1, v0}, Landroid/util/TypedValue;->applyDimension(IFLandroid/util/DisplayMetrics;)F

    move-result p1

    float-to-int p1, p1

    return p1
.end method

.method private finishSession()V
    .locals 3

    .line 258
    new-instance v0, Lcom/dragonis/checklist/Session;

    invoke-direct {v0}, Lcom/dragonis/checklist/Session;-><init>()V

    .line 259
    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v1, v1, Lcom/dragonis/checklist/Progress;->date:Ljava/lang/String;

    iput-object v1, v0, Lcom/dragonis/checklist/Session;->date:Ljava/lang/String;

    .line 260
    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v1, v1, Lcom/dragonis/checklist/Progress;->beforePhotos:Ljava/util/List;

    iput-object v1, v0, Lcom/dragonis/checklist/Session;->beforePhotos:Ljava/util/List;

    .line 261
    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v1, v1, Lcom/dragonis/checklist/Progress;->afterPhotos:Ljava/util/List;

    iput-object v1, v0, Lcom/dragonis/checklist/Session;->afterPhotos:Ljava/util/List;

    .line 262
    new-instance v1, Ljava/util/ArrayList;

    iget-object v2, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v2, v2, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-direct {v1, v2}, Ljava/util/ArrayList;-><init>(Ljava/util/Collection;)V

    iput-object v1, v0, Lcom/dragonis/checklist/Session;->checklistSnapshot:Ljava/util/List;

    .line 263
    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v1, v1, Lcom/dragonis/checklist/ChecklistItem;->history:Ljava/util/List;

    invoke-interface {v1, v0}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    .line 264
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    const/4 v1, 0x0

    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    .line 265
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->persist()V

    .line 266
    const-string v0, "Session termin\u00e9e \u2705"

    const/4 v1, 0x1

    invoke-static {p0, v0, v1}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object v0

    invoke-virtual {v0}, Landroid/widget/Toast;->show()V

    .line 267
    invoke-virtual {p0}, Lcom/dragonis/checklist/SessionActivity;->finish()V

    .line 268
    return-void
.end method

.method private initProgress()V
    .locals 5

    .line 85
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v0, v0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    if-nez v0, :cond_1

    .line 86
    new-instance v0, Lcom/dragonis/checklist/Progress;

    invoke-direct {v0}, Lcom/dragonis/checklist/Progress;-><init>()V

    .line 87
    invoke-static {}, Lcom/dragonis/checklist/DateUtil;->today()Ljava/lang/String;

    move-result-object v1

    iput-object v1, v0, Lcom/dragonis/checklist/Progress;->date:Ljava/lang/String;

    .line 88
    const-string v1, "BEFORE"

    iput-object v1, v0, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    .line 89
    const/4 v1, 0x0

    const/4 v2, 0x0

    :goto_0
    iget-object v3, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v3, v3, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-interface {v3}, Ljava/util/List;->size()I

    move-result v3

    if-ge v2, v3, :cond_0

    iget-object v3, v0, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-static {v1}, Ljava/lang/Boolean;->valueOf(Z)Ljava/lang/Boolean;

    move-result-object v4

    invoke-interface {v3, v4}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    add-int/lit8 v2, v2, 0x1

    goto :goto_0

    .line 90
    :cond_0
    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iput-object v0, v1, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    .line 92
    :cond_1
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v0, v0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    iput-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    .line 93
    return-void
.end method

.method private synthetic lambda$onCreate$0(Landroid/view/View;)V
    .locals 0

    .line 59
    invoke-virtual {p0}, Lcom/dragonis/checklist/SessionActivity;->finish()V

    return-void
.end method

.method private synthetic lambda$onCreate$1(Landroid/view/View;)V
    .locals 0

    .line 60
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->takePhoto()V

    return-void
.end method

.method private synthetic lambda$onCreate$2(Landroid/view/View;)V
    .locals 0

    .line 61
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->onPhotosDoneClicked()V

    return-void
.end method

.method private synthetic lambda$onCreate$3(Landroid/view/View;)V
    .locals 0

    .line 62
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->onChecklistDoneClicked()V

    return-void
.end method

.method private synthetic lambda$renderChecklist$4(ILandroid/widget/CompoundButton;Z)V
    .locals 1

    .line 165
    nop

    :goto_0
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p2, p2, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-interface {p2}, Ljava/util/List;->size()I

    move-result p2

    if-gt p2, p1, :cond_0

    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p2, p2, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    const/4 v0, 0x0

    invoke-static {v0}, Ljava/lang/Boolean;->valueOf(Z)Ljava/lang/Boolean;

    move-result-object v0

    invoke-interface {p2, v0}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    goto :goto_0

    .line 166
    :cond_0
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p2, p2, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-static {p3}, Ljava/lang/Boolean;->valueOf(Z)Ljava/lang/Boolean;

    move-result-object p3

    invoke-interface {p2, p1, p3}, Ljava/util/List;->set(ILjava/lang/Object;)Ljava/lang/Object;

    .line 167
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->persist()V

    .line 168
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->updateChecklistDoneButtonState()V

    .line 169
    return-void
.end method

.method private onChecklistDoneClicked()V
    .locals 2

    .line 200
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->allChecked()Z

    move-result v0

    if-nez v0, :cond_0

    .line 201
    const-string v0, "Coche toutes les \u00e9tapes de la check-list"

    const/4 v1, 0x0

    invoke-static {p0, v0, v1}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object v0

    invoke-virtual {v0}, Landroid/widget/Toast;->show()V

    .line 202
    return-void

    .line 204
    :cond_0
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    const-string v1, "AFTER"

    iput-object v1, v0, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    .line 205
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->persist()V

    .line 206
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->renderPhase()V

    .line 207
    return-void
.end method

.method private onPhotosDoneClicked()V
    .locals 2

    .line 189
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->beforePhotos:Ljava/util/List;

    invoke-interface {v0}, Ljava/util/List;->isEmpty()Z

    move-result v0

    if-eqz v0, :cond_0

    .line 190
    const-string v0, "Prends au moins une photo avant de continuer"

    const/4 v1, 0x0

    invoke-static {p0, v0, v1}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object v0

    invoke-virtual {v0}, Landroid/widget/Toast;->show()V

    .line 191
    return-void

    .line 193
    :cond_0
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v1, v1, Lcom/dragonis/checklist/Progress;->beforePhotos:Ljava/util/List;

    invoke-interface {v1}, Ljava/util/List;->size()I

    move-result v1

    iput v1, v0, Lcom/dragonis/checklist/Progress;->targetCount:I

    .line 194
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    const-string v1, "CHECKLIST"

    iput-object v1, v0, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    .line 195
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->persist()V

    .line 196
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->renderPhase()V

    .line 197
    return-void
.end method

.method private persist()V
    .locals 1

    .line 96
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->store:Lcom/dragonis/checklist/Store;

    invoke-virtual {v0}, Lcom/dragonis/checklist/Store;->save()V

    .line 97
    return-void
.end method

.method private renderChecklist()V
    .locals 6

    .line 155
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->containerChecklistCheck:Landroid/widget/LinearLayout;

    invoke-virtual {v0}, Landroid/widget/LinearLayout;->removeAllViews()V

    .line 156
    const/4 v0, 0x0

    const/4 v1, 0x0

    :goto_0
    iget-object v2, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v2, v2, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-interface {v2}, Ljava/util/List;->size()I

    move-result v2

    if-ge v1, v2, :cond_1

    .line 157
    nop

    .line 158
    invoke-static {p0}, Landroid/view/LayoutInflater;->from(Landroid/content/Context;)Landroid/view/LayoutInflater;

    move-result-object v2

    const v3, 0x7f040004

    iget-object v4, p0, Lcom/dragonis/checklist/SessionActivity;->containerChecklistCheck:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v3, v4, v0}, Landroid/view/LayoutInflater;->inflate(ILandroid/view/ViewGroup;Z)Landroid/view/View;

    move-result-object v2

    .line 159
    const v3, 0x7f030018

    invoke-virtual {v2, v3}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object v3

    check-cast v3, Landroid/widget/TextView;

    .line 160
    const v4, 0x7f03000a

    invoke-virtual {v2, v4}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object v4

    check-cast v4, Landroid/widget/CheckBox;

    .line 161
    iget-object v5, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v5, v5, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-interface {v5, v1}, Ljava/util/List;->get(I)Ljava/lang/Object;

    move-result-object v5

    check-cast v5, Ljava/lang/CharSequence;

    invoke-virtual {v3, v5}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 162
    iget-object v3, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v3, v3, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-interface {v3}, Ljava/util/List;->size()I

    move-result v3

    if-ge v1, v3, :cond_0

    iget-object v3, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v3, v3, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-interface {v3, v1}, Ljava/util/List;->get(I)Ljava/lang/Object;

    move-result-object v3

    check-cast v3, Ljava/lang/Boolean;

    invoke-virtual {v3}, Ljava/lang/Boolean;->booleanValue()Z

    move-result v3

    if-eqz v3, :cond_0

    const/4 v3, 0x1

    goto :goto_1

    :cond_0
    const/4 v3, 0x0

    .line 163
    :goto_1
    invoke-virtual {v4, v3}, Landroid/widget/CheckBox;->setChecked(Z)V

    .line 164
    new-instance v3, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda0;

    invoke-direct {v3, p0, v1}, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda0;-><init>(Lcom/dragonis/checklist/SessionActivity;I)V

    invoke-virtual {v4, v3}, Landroid/widget/CheckBox;->setOnCheckedChangeListener(Landroid/widget/CompoundButton$OnCheckedChangeListener;)V

    .line 170
    iget-object v3, p0, Lcom/dragonis/checklist/SessionActivity;->containerChecklistCheck:Landroid/widget/LinearLayout;

    invoke-virtual {v3, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 156
    add-int/lit8 v1, v1, 0x1

    goto :goto_0

    .line 172
    :cond_1
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->updateChecklistDoneButtonState()V

    .line 173
    return-void
.end method

.method private renderPhase()V
    .locals 5

    .line 106
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->panelPhotos:Landroid/widget/LinearLayout;

    const/16 v1, 0x8

    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->setVisibility(I)V

    .line 107
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->panelChecklist:Landroid/widget/LinearLayout;

    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->setVisibility(I)V

    .line 109
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    invoke-virtual {v0}, Ljava/lang/String;->hashCode()I

    move-result v2

    const/4 v3, 0x0

    const/4 v4, -0x1

    sparse-switch v2, :sswitch_data_0

    :cond_0
    goto :goto_0

    :sswitch_0
    const-string v2, "BEFORE"

    invoke-virtual {v0, v2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_0

    const/4 v0, 0x0

    goto :goto_1

    :sswitch_1
    const-string v2, "AFTER"

    invoke-virtual {v0, v2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_0

    const/4 v0, 0x2

    goto :goto_1

    :sswitch_2
    const-string v2, "CHECKLIST"

    invoke-virtual {v0, v2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_0

    const/4 v0, 0x1

    goto :goto_1

    :goto_0
    const/4 v0, -0x1

    :goto_1
    packed-switch v0, :pswitch_data_0

    goto :goto_2

    .line 122
    :pswitch_0
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->txtStep:Landroid/widget/TextView;

    const-string v2, "\u00c9tape 3/3 \u00b7 Photos apr\u00e8s"

    invoke-virtual {v0, v2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 123
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->panelPhotos:Landroid/widget/LinearLayout;

    invoke-virtual {v0, v3}, Landroid/widget/LinearLayout;->setVisibility(I)V

    .line 124
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->btnPhotosDone:Landroid/widget/TextView;

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setVisibility(I)V

    .line 125
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->afterPhotos:Ljava/util/List;

    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget v1, v1, Lcom/dragonis/checklist/Progress;->targetCount:I

    invoke-direct {p0, v0, v1}, Lcom/dragonis/checklist/SessionActivity;->renderPhotoPanel(Ljava/util/List;I)V

    goto :goto_2

    .line 117
    :pswitch_1
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->txtStep:Landroid/widget/TextView;

    const-string v1, "\u00c9tape 2/3 \u00b7 Check-list"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 118
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->panelChecklist:Landroid/widget/LinearLayout;

    invoke-virtual {v0, v3}, Landroid/widget/LinearLayout;->setVisibility(I)V

    .line 119
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->renderChecklist()V

    .line 120
    goto :goto_2

    .line 111
    :pswitch_2
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->txtStep:Landroid/widget/TextView;

    const-string v1, "\u00c9tape 1/3 \u00b7 Photos avant"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 112
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->panelPhotos:Landroid/widget/LinearLayout;

    invoke-virtual {v0, v3}, Landroid/widget/LinearLayout;->setVisibility(I)V

    .line 113
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->btnPhotosDone:Landroid/widget/TextView;

    invoke-virtual {v0, v3}, Landroid/widget/TextView;->setVisibility(I)V

    .line 114
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->beforePhotos:Ljava/util/List;

    invoke-direct {p0, v0, v4}, Lcom/dragonis/checklist/SessionActivity;->renderPhotoPanel(Ljava/util/List;I)V

    .line 115
    nop

    .line 128
    :goto_2
    return-void

    :sswitch_data_0
    .sparse-switch
        -0x421b1e7a -> :sswitch_2
        0x3b50dbc -> :sswitch_1
        0x748d337f -> :sswitch_0
    .end sparse-switch

    :pswitch_data_0
    .packed-switch 0x0
        :pswitch_2
        :pswitch_1
        :pswitch_0
    .end packed-switch
.end method

.method private renderPhotoPanel(Ljava/util/List;I)V
    .locals 6
    .annotation system Ldalvik/annotation/Signature;
        value = {
            "(",
            "Ljava/util/List<",
            "Ljava/lang/String;",
            ">;I)V"
        }
    .end annotation

    .line 131
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->containerPhotos:Landroid/widget/LinearLayout;

    invoke-virtual {v0}, Landroid/widget/LinearLayout;->removeAllViews()V

    .line 132
    const/16 v0, 0x5a

    invoke-direct {p0, v0}, Lcom/dragonis/checklist/SessionActivity;->dp(I)I

    move-result v0

    .line 133
    const/16 v1, 0x8

    invoke-direct {p0, v1}, Lcom/dragonis/checklist/SessionActivity;->dp(I)I

    move-result v1

    .line 134
    invoke-interface {p1}, Ljava/util/List;->iterator()Ljava/util/Iterator;

    move-result-object v2

    :goto_0
    invoke-interface {v2}, Ljava/util/Iterator;->hasNext()Z

    move-result v3

    if-eqz v3, :cond_0

    invoke-interface {v2}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v3

    check-cast v3, Ljava/lang/String;

    .line 135
    new-instance v4, Landroid/widget/ImageView;

    invoke-direct {v4, p0}, Landroid/widget/ImageView;-><init>(Landroid/content/Context;)V

    .line 136
    new-instance v5, Landroid/widget/LinearLayout$LayoutParams;

    invoke-direct {v5, v0, v0}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    .line 137
    invoke-virtual {v5, v1, v1, v1, v1}, Landroid/widget/LinearLayout$LayoutParams;->setMargins(IIII)V

    .line 138
    invoke-virtual {v4, v5}, Landroid/widget/ImageView;->setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V

    .line 139
    sget-object v5, Landroid/widget/ImageView$ScaleType;->CENTER_CROP:Landroid/widget/ImageView$ScaleType;

    invoke-virtual {v4, v5}, Landroid/widget/ImageView;->setScaleType(Landroid/widget/ImageView$ScaleType;)V

    .line 140
    const v5, 0x7f020005

    invoke-virtual {v4, v5}, Landroid/widget/ImageView;->setBackgroundResource(I)V

    .line 141
    invoke-static {p0, v3, v4, v0}, Lcom/dragonis/checklist/ImageUtil;->loadThumbnail(Landroid/content/Context;Ljava/lang/String;Landroid/widget/ImageView;I)V

    .line 142
    iget-object v3, p0, Lcom/dragonis/checklist/SessionActivity;->containerPhotos:Landroid/widget/LinearLayout;

    invoke-virtual {v3, v4}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 143
    goto :goto_0

    .line 145
    :cond_0
    if-gez p2, :cond_2

    .line 146
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->txtPhotoCounter:Landroid/widget/TextView;

    invoke-interface {p1}, Ljava/util/List;->size()I

    move-result v0

    new-instance v1, Ljava/lang/StringBuilder;

    invoke-direct {v1}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v1, v0}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v1, " photo(s) prise(s)"

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    invoke-virtual {p2, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 147
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->btnPhotosDone:Landroid/widget/TextView;

    invoke-interface {p1}, Ljava/util/List;->size()I

    move-result v0

    new-instance v1, Ljava/lang/StringBuilder;

    invoke-direct {v1}, Ljava/lang/StringBuilder;-><init>()V

    const-string v2, "Terminer les photos ("

    invoke-virtual {v1, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v1

    invoke-virtual {v1, v0}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v1, ")"

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    invoke-virtual {p2, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 148
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->btnPhotosDone:Landroid/widget/TextView;

    invoke-interface {p1}, Ljava/util/List;->isEmpty()Z

    move-result p1

    if-eqz p1, :cond_1

    const/high16 p1, 0x3f000000    # 0.5f

    goto :goto_1

    :cond_1
    const/high16 p1, 0x3f800000    # 1.0f

    :goto_1
    invoke-virtual {p2, p1}, Landroid/widget/TextView;->setAlpha(F)V

    goto :goto_2

    .line 150
    :cond_2
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->txtPhotoCounter:Landroid/widget/TextView;

    invoke-interface {p1}, Ljava/util/List;->size()I

    move-result p1

    new-instance v1, Ljava/lang/StringBuilder;

    invoke-direct {v1}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v1, p1}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object p1

    const-string v1, " / "

    invoke-virtual {p1, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p1

    invoke-virtual {p1, p2}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object p1

    const-string p2, " photos"

    invoke-virtual {p1, p2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p1

    invoke-virtual {v0, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 152
    :goto_2
    return-void
.end method

.method private takePhoto()V
    .locals 3

    .line 210
    sget v0, Landroid/os/Build$VERSION;->SDK_INT:I

    const/16 v1, 0x17

    if-lt v0, v1, :cond_0

    .line 211
    const-string v0, "android.permission.CAMERA"

    invoke-virtual {p0, v0}, Lcom/dragonis/checklist/SessionActivity;->checkSelfPermission(Ljava/lang/String;)I

    move-result v1

    if-eqz v1, :cond_0

    .line 212
    filled-new-array {v0}, [Ljava/lang/String;

    move-result-object v0

    const/16 v1, 0x29

    invoke-virtual {p0, v0, v1}, Lcom/dragonis/checklist/SessionActivity;->requestPermissions([Ljava/lang/String;I)V

    .line 213
    return-void

    .line 215
    :cond_0
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    const-string v1, "AFTER"

    invoke-virtual {v1, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_1

    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->afterPhotos:Ljava/util/List;

    invoke-interface {v0}, Ljava/util/List;->size()I

    move-result v0

    iget-object v2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget v2, v2, Lcom/dragonis/checklist/Progress;->targetCount:I

    if-lt v0, v2, :cond_1

    .line 216
    return-void

    .line 218
    :cond_1
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    invoke-virtual {v1, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_2

    const-string v0, "after"

    goto :goto_0

    :cond_2
    const-string v0, "before"

    :goto_0
    invoke-static {p0, v0}, Lcom/dragonis/checklist/ImageUtil;->createNewImageUri(Landroid/content/Context;Ljava/lang/String;)Landroid/net/Uri;

    move-result-object v0

    iput-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->pendingPhotoUri:Landroid/net/Uri;

    .line 219
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->pendingPhotoUri:Landroid/net/Uri;

    if-nez v0, :cond_3

    .line 220
    const-string v0, "Impossible de cr\u00e9er le fichier photo"

    const/4 v1, 0x0

    invoke-static {p0, v0, v1}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object v0

    invoke-virtual {v0}, Landroid/widget/Toast;->show()V

    .line 221
    return-void

    .line 223
    :cond_3
    new-instance v0, Landroid/content/Intent;

    const-string v1, "android.media.action.IMAGE_CAPTURE"

    invoke-direct {v0, v1}, Landroid/content/Intent;-><init>(Ljava/lang/String;)V

    .line 224
    const-string v1, "output"

    iget-object v2, p0, Lcom/dragonis/checklist/SessionActivity;->pendingPhotoUri:Landroid/net/Uri;

    invoke-virtual {v0, v1, v2}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;

    .line 225
    const/16 v1, 0x28

    invoke-virtual {p0, v0, v1}, Lcom/dragonis/checklist/SessionActivity;->startActivityForResult(Landroid/content/Intent;I)V

    .line 226
    return-void
.end method

.method private updateChecklistDoneButtonState()V
    .locals 2

    .line 182
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->allChecked()Z

    move-result v0

    .line 183
    iget-object v1, p0, Lcom/dragonis/checklist/SessionActivity;->btnChecklistDone:Landroid/widget/TextView;

    if-eqz v0, :cond_0

    const/high16 v0, 0x3f800000    # 1.0f

    goto :goto_0

    :cond_0
    const/high16 v0, 0x3f000000    # 0.5f

    :goto_0
    invoke-virtual {v1, v0}, Landroid/widget/TextView;->setAlpha(F)V

    .line 184
    return-void
.end method


# virtual methods
.method protected onActivityResult(IILandroid/content/Intent;)V
    .locals 0

    .line 239
    invoke-super {p0, p1, p2, p3}, Landroid/app/Activity;->onActivityResult(IILandroid/content/Intent;)V

    .line 240
    const/16 p3, 0x28

    if-ne p1, p3, :cond_4

    const/4 p1, -0x1

    if-ne p2, p1, :cond_4

    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->pendingPhotoUri:Landroid/net/Uri;

    if-nez p1, :cond_0

    goto :goto_2

    .line 242
    :cond_0
    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->pendingPhotoUri:Landroid/net/Uri;

    invoke-virtual {p1}, Landroid/net/Uri;->toString()Ljava/lang/String;

    move-result-object p1

    .line 243
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p2, p2, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    const-string p3, "BEFORE"

    invoke-virtual {p3, p2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result p2

    const-string p3, "AFTER"

    if-eqz p2, :cond_1

    .line 244
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p2, p2, Lcom/dragonis/checklist/Progress;->beforePhotos:Ljava/util/List;

    invoke-interface {p2, p1}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    goto :goto_0

    .line 245
    :cond_1
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p2, p2, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    invoke-virtual {p3, p2}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result p2

    if-eqz p2, :cond_2

    .line 246
    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p2, p2, Lcom/dragonis/checklist/Progress;->afterPhotos:Ljava/util/List;

    invoke-interface {p2, p1}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    .line 248
    :cond_2
    :goto_0
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->persist()V

    .line 250
    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p1, p1, Lcom/dragonis/checklist/Progress;->phase:Ljava/lang/String;

    invoke-virtual {p3, p1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result p1

    if-eqz p1, :cond_3

    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget-object p1, p1, Lcom/dragonis/checklist/Progress;->afterPhotos:Ljava/util/List;

    invoke-interface {p1}, Ljava/util/List;->size()I

    move-result p1

    iget-object p2, p0, Lcom/dragonis/checklist/SessionActivity;->progress:Lcom/dragonis/checklist/Progress;

    iget p2, p2, Lcom/dragonis/checklist/Progress;->targetCount:I

    if-lt p1, p2, :cond_3

    .line 251
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->finishSession()V

    goto :goto_1

    .line 253
    :cond_3
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->renderPhase()V

    .line 255
    :goto_1
    return-void

    .line 240
    :cond_4
    :goto_2
    return-void
.end method

.method protected onCreate(Landroid/os/Bundle;)V
    .locals 1

    .line 36
    invoke-super {p0, p1}, Landroid/app/Activity;->onCreate(Landroid/os/Bundle;)V

    .line 37
    const p1, 0x7f040002

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->setContentView(I)V

    .line 38
    invoke-static {p0}, Lcom/dragonis/checklist/Store;->get(Landroid/content/Context;)Lcom/dragonis/checklist/Store;

    move-result-object p1

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->store:Lcom/dragonis/checklist/Store;

    .line 40
    invoke-virtual {p0}, Lcom/dragonis/checklist/SessionActivity;->getIntent()Landroid/content/Intent;

    move-result-object p1

    const-string v0, "item_id"

    invoke-virtual {p1, v0}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    .line 41
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->store:Lcom/dragonis/checklist/Store;

    invoke-virtual {v0, p1}, Lcom/dragonis/checklist/Store;->findById(Ljava/lang/String;)Lcom/dragonis/checklist/ChecklistItem;

    move-result-object p1

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    .line 42
    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    if-nez p1, :cond_0

    .line 43
    invoke-virtual {p0}, Lcom/dragonis/checklist/SessionActivity;->finish()V

    .line 44
    return-void

    .line 47
    :cond_0
    const p1, 0x7f03001a

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/TextView;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->txtTitle:Landroid/widget/TextView;

    .line 48
    const p1, 0x7f030019

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/TextView;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->txtStep:Landroid/widget/TextView;

    .line 49
    const p1, 0x7f030014

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/LinearLayout;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->panelPhotos:Landroid/widget/LinearLayout;

    .line 50
    const p1, 0x7f030013

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/LinearLayout;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->panelChecklist:Landroid/widget/LinearLayout;

    .line 51
    const p1, 0x7f030017

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/TextView;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->txtPhotoCounter:Landroid/widget/TextView;

    .line 52
    const p1, 0x7f03000d

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/LinearLayout;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->containerPhotos:Landroid/widget/LinearLayout;

    .line 53
    const p1, 0x7f030008

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/ImageView;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->btnTakePhoto:Landroid/widget/ImageView;

    .line 54
    const p1, 0x7f030004

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/TextView;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->btnPhotosDone:Landroid/widget/TextView;

    .line 55
    const p1, 0x7f03000c

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/LinearLayout;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->containerChecklistCheck:Landroid/widget/LinearLayout;

    .line 56
    const p1, 0x7f030002

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/TextView;

    iput-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->btnChecklistDone:Landroid/widget/TextView;

    .line 58
    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->txtTitle:Landroid/widget/TextView;

    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v0, v0, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 59
    const p1, 0x7f030001

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/SessionActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    new-instance v0, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda1;

    invoke-direct {v0, p0}, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda1;-><init>(Lcom/dragonis/checklist/SessionActivity;)V

    invoke-virtual {p1, v0}, Landroid/view/View;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 60
    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->btnTakePhoto:Landroid/widget/ImageView;

    new-instance v0, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda2;

    invoke-direct {v0, p0}, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda2;-><init>(Lcom/dragonis/checklist/SessionActivity;)V

    invoke-virtual {p1, v0}, Landroid/widget/ImageView;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 61
    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->btnPhotosDone:Landroid/widget/TextView;

    new-instance v0, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda3;

    invoke-direct {v0, p0}, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda3;-><init>(Lcom/dragonis/checklist/SessionActivity;)V

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 62
    iget-object p1, p0, Lcom/dragonis/checklist/SessionActivity;->btnChecklistDone:Landroid/widget/TextView;

    new-instance v0, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda4;

    invoke-direct {v0, p0}, Lcom/dragonis/checklist/SessionActivity$$ExternalSyntheticLambda4;-><init>(Lcom/dragonis/checklist/SessionActivity;)V

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 64
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->initProgress()V

    .line 65
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->renderPhase()V

    .line 66
    return-void
.end method

.method protected onPause()V
    .locals 0

    .line 79
    invoke-super {p0}, Landroid/app/Activity;->onPause()V

    .line 81
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->persist()V

    .line 82
    return-void
.end method

.method public onRequestPermissionsResult(I[Ljava/lang/String;[I)V
    .locals 0

    .line 230
    invoke-super {p0, p1, p2, p3}, Landroid/app/Activity;->onRequestPermissionsResult(I[Ljava/lang/String;[I)V

    .line 231
    const/16 p2, 0x29

    if-ne p1, p2, :cond_0

    array-length p1, p3

    if-lez p1, :cond_0

    const/4 p1, 0x0

    aget p1, p3, p1

    if-nez p1, :cond_0

    .line 233
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->takePhoto()V

    .line 235
    :cond_0
    return-void
.end method

.method protected onResume()V
    .locals 1

    .line 70
    invoke-super {p0}, Landroid/app/Activity;->onResume()V

    .line 72
    iget-object v0, p0, Lcom/dragonis/checklist/SessionActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    invoke-virtual {v0}, Lcom/dragonis/checklist/ChecklistItem;->purgeStaleProgress()V

    .line 73
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->initProgress()V

    .line 74
    invoke-direct {p0}, Lcom/dragonis/checklist/SessionActivity;->renderPhase()V

    .line 75
    return-void
.end method
