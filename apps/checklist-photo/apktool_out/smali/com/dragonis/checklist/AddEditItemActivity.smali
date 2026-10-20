.class public Lcom/dragonis/checklist/AddEditItemActivity;
.super Landroid/app/Activity;
.source "AddEditItemActivity.java"


# static fields
.field private static final REQ_CAMERA_ICON:I = 0xa

.field private static final REQ_CAMERA_PERM:I = 0x14

.field private static final REQ_GALLERY_ICON:I = 0xb


# instance fields
.field private btnDelete:Landroid/widget/TextView;

.field private containerChecklist:Landroid/widget/LinearLayout;

.field private editName:Landroid/widget/EditText;

.field private imgPreview:Landroid/widget/ImageView;

.field private isNew:Z

.field private item:Lcom/dragonis/checklist/ChecklistItem;

.field private pendingCameraUri:Landroid/net/Uri;

.field private final rowViews:Ljava/util/List;
    .annotation system Ldalvik/annotation/Signature;
        value = {
            "Ljava/util/List<",
            "Landroid/view/View;",
            ">;"
        }
    .end annotation
.end field

.field private store:Lcom/dragonis/checklist/Store;


# direct methods
.method public static synthetic $r8$lambda$8gqftIQ3bIBiBOBdhX1p9of0RjI(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$onCreate$2(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$Rsf5F9KoKee4tQcTiXwrwX2i1IU(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$onCreate$3(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$Rz3JVuAHZDhTtERYRQ_3A4hyw9Y(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$onCreate$1(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$YN1q8Q5XvSV9qesS0LaxYgN3bdg(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/content/DialogInterface;I)V
    .locals 0

    invoke-direct {p0, p1, p2}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$showIconPickerDialog$5(Landroid/content/DialogInterface;I)V

    return-void
.end method

.method public static synthetic $r8$lambda$f4KaDAkaYYFSw3M63ArZ0niGL8Y(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$onCreate$0(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$n8_Xxt8X08kzzKbObuyJYyYidE8(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/content/DialogInterface;I)V
    .locals 0

    invoke-direct {p0, p1, p2}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$confirmDelete$7(Landroid/content/DialogInterface;I)V

    return-void
.end method

.method public static synthetic $r8$lambda$q0gVu_rweL0eUTjhRMjYqZW4mwg(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$onCreate$4(Landroid/view/View;)V

    return-void
.end method

.method public static synthetic $r8$lambda$zJaSz3m-Nj1_PjX09exb9ljT9rc(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/view/View;Landroid/view/View;)V
    .locals 0

    invoke-direct {p0, p1, p2}, Lcom/dragonis/checklist/AddEditItemActivity;->lambda$addChecklistRow$6(Landroid/view/View;Landroid/view/View;)V

    return-void
.end method

.method public constructor <init>()V
    .locals 1

    .line 23
    invoke-direct {p0}, Landroid/app/Activity;-><init>()V

    .line 39
    new-instance v0, Ljava/util/ArrayList;

    invoke-direct {v0}, Ljava/util/ArrayList;-><init>()V

    iput-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->rowViews:Ljava/util/List;

    return-void
.end method

.method private addChecklistRow(Ljava/lang/String;)V
    .locals 4

    .line 144
    invoke-static {p0}, Landroid/view/LayoutInflater;->from(Landroid/content/Context;)Landroid/view/LayoutInflater;

    move-result-object v0

    iget-object v1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->containerChecklist:Landroid/widget/LinearLayout;

    const/4 v2, 0x0

    const v3, 0x7f040005

    invoke-virtual {v0, v3, v1, v2}, Landroid/view/LayoutInflater;->inflate(ILandroid/view/ViewGroup;Z)Landroid/view/View;

    move-result-object v0

    .line 145
    const v1, 0x7f03000f

    invoke-virtual {v0, v1}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object v1

    check-cast v1, Landroid/widget/EditText;

    .line 146
    invoke-virtual {v1, p1}, Landroid/widget/EditText;->setText(Ljava/lang/CharSequence;)V

    .line 147
    const p1, 0x7f030006

    invoke-virtual {v0, p1}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object p1

    new-instance v1, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda2;

    invoke-direct {v1, p0, v0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda2;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;Landroid/view/View;)V

    invoke-virtual {p1, v1}, Landroid/view/View;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 151
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->containerChecklist:Landroid/widget/LinearLayout;

    invoke-virtual {p1, v0}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 152
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->rowViews:Ljava/util/List;

    invoke-interface {p1, v0}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    .line 153
    return-void
.end method

.method private confirmDelete()V
    .locals 4

    .line 188
    new-instance v0, Landroid/app/AlertDialog$Builder;

    invoke-direct {v0, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    iget-object v1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v1, v1, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    new-instance v2, Ljava/lang/StringBuilder;

    invoke-direct {v2}, Ljava/lang/StringBuilder;-><init>()V

    const-string v3, "Supprimer \u00ab "

    invoke-virtual {v2, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    invoke-virtual {v2, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v1

    const-string v2, " \u00bb ?"

    invoke-virtual {v1, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v1

    invoke-virtual {v1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v1

    .line 189
    invoke-virtual {v0, v1}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    .line 190
    const-string v1, "La check-list et la progression du jour seront supprim\u00e9es. L\'historique des photos pass\u00e9es reste sur l\'appareil."

    invoke-virtual {v0, v1}, Landroid/app/AlertDialog$Builder;->setMessage(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    new-instance v1, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda1;

    invoke-direct {v1, p0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda1;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;)V

    .line 191
    const-string v2, "Supprimer"

    invoke-virtual {v0, v2, v1}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    .line 195
    const-string v1, "Annuler"

    const/4 v2, 0x0

    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    .line 196
    invoke-virtual {v0}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 197
    return-void
.end method

.method private synthetic lambda$addChecklistRow$6(Landroid/view/View;Landroid/view/View;)V
    .locals 0

    .line 148
    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->containerChecklist:Landroid/widget/LinearLayout;

    invoke-virtual {p2, p1}, Landroid/widget/LinearLayout;->removeView(Landroid/view/View;)V

    .line 149
    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->rowViews:Ljava/util/List;

    invoke-interface {p2, p1}, Ljava/util/List;->remove(Ljava/lang/Object;)Z

    .line 150
    return-void
.end method

.method private synthetic lambda$confirmDelete$7(Landroid/content/DialogInterface;I)V
    .locals 0

    .line 192
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->store:Lcom/dragonis/checklist/Store;

    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p2, p2, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    invoke-virtual {p1, p2}, Lcom/dragonis/checklist/Store;->removeItem(Ljava/lang/String;)V

    .line 193
    invoke-virtual {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->finish()V

    .line 194
    return-void
.end method

.method private synthetic lambda$onCreate$0(Landroid/view/View;)V
    .locals 0

    .line 63
    invoke-virtual {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->finish()V

    return-void
.end method

.method private synthetic lambda$onCreate$1(Landroid/view/View;)V
    .locals 0

    .line 64
    invoke-direct {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->showIconPickerDialog()V

    return-void
.end method

.method private synthetic lambda$onCreate$2(Landroid/view/View;)V
    .locals 0

    .line 65
    const-string p1, ""

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->addChecklistRow(Ljava/lang/String;)V

    return-void
.end method

.method private synthetic lambda$onCreate$3(Landroid/view/View;)V
    .locals 0

    .line 66
    invoke-direct {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->save()V

    return-void
.end method

.method private synthetic lambda$onCreate$4(Landroid/view/View;)V
    .locals 0

    .line 79
    invoke-direct {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->confirmDelete()V

    return-void
.end method

.method private synthetic lambda$showIconPickerDialog$5(Landroid/content/DialogInterface;I)V
    .locals 0

    .line 86
    if-nez p2, :cond_0

    invoke-direct {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->launchCamera()V

    goto :goto_0

    .line 87
    :cond_0
    invoke-direct {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->launchGallery()V

    .line 88
    :goto_0
    return-void
.end method

.method private launchCamera()V
    .locals 3

    .line 93
    sget v0, Landroid/os/Build$VERSION;->SDK_INT:I

    const/16 v1, 0x17

    if-lt v0, v1, :cond_0

    .line 94
    const-string v0, "android.permission.CAMERA"

    invoke-virtual {p0, v0}, Lcom/dragonis/checklist/AddEditItemActivity;->checkSelfPermission(Ljava/lang/String;)I

    move-result v1

    if-eqz v1, :cond_0

    .line 95
    filled-new-array {v0}, [Ljava/lang/String;

    move-result-object v0

    const/16 v1, 0x14

    invoke-virtual {p0, v0, v1}, Lcom/dragonis/checklist/AddEditItemActivity;->requestPermissions([Ljava/lang/String;I)V

    .line 96
    return-void

    .line 98
    :cond_0
    const-string v0, "icon"

    invoke-static {p0, v0}, Lcom/dragonis/checklist/ImageUtil;->createNewImageUri(Landroid/content/Context;Ljava/lang/String;)Landroid/net/Uri;

    move-result-object v0

    iput-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->pendingCameraUri:Landroid/net/Uri;

    .line 99
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->pendingCameraUri:Landroid/net/Uri;

    if-nez v0, :cond_1

    .line 100
    const-string v0, "Impossible de cr\u00e9er le fichier photo"

    const/4 v1, 0x0

    invoke-static {p0, v0, v1}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object v0

    invoke-virtual {v0}, Landroid/widget/Toast;->show()V

    .line 101
    return-void

    .line 103
    :cond_1
    new-instance v0, Landroid/content/Intent;

    const-string v1, "android.media.action.IMAGE_CAPTURE"

    invoke-direct {v0, v1}, Landroid/content/Intent;-><init>(Ljava/lang/String;)V

    .line 104
    const-string v1, "output"

    iget-object v2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->pendingCameraUri:Landroid/net/Uri;

    invoke-virtual {v0, v1, v2}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;

    .line 105
    const/16 v1, 0xa

    invoke-virtual {p0, v0, v1}, Lcom/dragonis/checklist/AddEditItemActivity;->startActivityForResult(Landroid/content/Intent;I)V

    .line 106
    return-void
.end method

.method private launchGallery()V
    .locals 2

    .line 109
    new-instance v0, Landroid/content/Intent;

    const-string v1, "android.intent.action.OPEN_DOCUMENT"

    invoke-direct {v0, v1}, Landroid/content/Intent;-><init>(Ljava/lang/String;)V

    .line 110
    const-string v1, "android.intent.category.OPENABLE"

    invoke-virtual {v0, v1}, Landroid/content/Intent;->addCategory(Ljava/lang/String;)Landroid/content/Intent;

    .line 111
    const-string v1, "image/*"

    invoke-virtual {v0, v1}, Landroid/content/Intent;->setType(Ljava/lang/String;)Landroid/content/Intent;

    .line 112
    const/4 v1, 0x1

    invoke-virtual {v0, v1}, Landroid/content/Intent;->addFlags(I)Landroid/content/Intent;

    .line 113
    const/16 v1, 0xb

    invoke-virtual {p0, v0, v1}, Lcom/dragonis/checklist/AddEditItemActivity;->startActivityForResult(Landroid/content/Intent;I)V

    .line 114
    return-void
.end method

.method private save()V
    .locals 6

    .line 156
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->editName:Landroid/widget/EditText;

    invoke-virtual {v0}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object v0

    .line 157
    invoke-static {v0}, Landroid/text/TextUtils;->isEmpty(Ljava/lang/CharSequence;)Z

    move-result v1

    const/4 v2, 0x0

    if-eqz v1, :cond_0

    .line 158
    const-string v0, "Donne un nom \u00e0 l\'\u00e9l\u00e9ment"

    invoke-static {p0, v0, v2}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object v0

    invoke-virtual {v0}, Landroid/widget/Toast;->show()V

    .line 159
    return-void

    .line 161
    :cond_0
    new-instance v1, Ljava/util/ArrayList;

    invoke-direct {v1}, Ljava/util/ArrayList;-><init>()V

    .line 162
    iget-object v3, p0, Lcom/dragonis/checklist/AddEditItemActivity;->rowViews:Ljava/util/List;

    invoke-interface {v3}, Ljava/util/List;->iterator()Ljava/util/Iterator;

    move-result-object v3

    :goto_0
    invoke-interface {v3}, Ljava/util/Iterator;->hasNext()Z

    move-result v4

    if-eqz v4, :cond_2

    invoke-interface {v3}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v4

    check-cast v4, Landroid/view/View;

    .line 163
    const v5, 0x7f03000f

    invoke-virtual {v4, v5}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object v4

    check-cast v4, Landroid/widget/EditText;

    .line 164
    invoke-virtual {v4}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object v4

    invoke-virtual {v4}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object v4

    invoke-virtual {v4}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object v4

    .line 165
    invoke-static {v4}, Landroid/text/TextUtils;->isEmpty(Ljava/lang/CharSequence;)Z

    move-result v5

    if-nez v5, :cond_1

    invoke-interface {v1, v4}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    .line 166
    :cond_1
    goto :goto_0

    .line 167
    :cond_2
    invoke-interface {v1}, Ljava/util/List;->isEmpty()Z

    move-result v3

    if-eqz v3, :cond_3

    .line 168
    const-string v0, "Ajoute au moins une \u00e9tape \u00e0 la check-list"

    invoke-static {p0, v0, v2}, Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;

    move-result-object v0

    invoke-virtual {v0}, Landroid/widget/Toast;->show()V

    .line 169
    return-void

    .line 172
    :cond_3
    iget-object v2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iput-object v0, v2, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    .line 173
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    .line 175
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v0, v0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    if-eqz v0, :cond_4

    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v0, v0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v0, v0, Lcom/dragonis/checklist/Progress;->checked:Ljava/util/List;

    invoke-interface {v0}, Ljava/util/List;->size()I

    move-result v0

    invoke-interface {v1}, Ljava/util/List;->size()I

    move-result v1

    if-eq v0, v1, :cond_4

    .line 176
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    const/4 v1, 0x0

    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    .line 179
    :cond_4
    iget-boolean v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->isNew:Z

    if-eqz v0, :cond_5

    .line 180
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->store:Lcom/dragonis/checklist/Store;

    iget-object v1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    invoke-virtual {v0, v1}, Lcom/dragonis/checklist/Store;->addItem(Lcom/dragonis/checklist/ChecklistItem;)V

    goto :goto_1

    .line 182
    :cond_5
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->store:Lcom/dragonis/checklist/Store;

    invoke-virtual {v0}, Lcom/dragonis/checklist/Store;->save()V

    .line 184
    :goto_1
    invoke-virtual {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->finish()V

    .line 185
    return-void
.end method

.method private showIconPickerDialog()V
    .locals 4

    .line 84
    new-instance v0, Landroid/app/AlertDialog$Builder;

    invoke-direct {v0, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const/4 v1, 0x2

    new-array v1, v1, [Ljava/lang/CharSequence;

    const/4 v2, 0x0

    const-string v3, "Prendre une photo"

    aput-object v3, v1, v2

    const/4 v2, 0x1

    const-string v3, "Choisir depuis la galerie"

    aput-object v3, v1, v2

    new-instance v2, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda0;

    invoke-direct {v2, p0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda0;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;)V

    .line 85
    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    .line 89
    invoke-virtual {v0}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 90
    return-void
.end method


# virtual methods
.method protected onActivityResult(IILandroid/content/Intent;)V
    .locals 1

    .line 127
    invoke-super {p0, p1, p2, p3}, Landroid/app/Activity;->onActivityResult(IILandroid/content/Intent;)V

    .line 128
    const/4 v0, -0x1

    if-eq p2, v0, :cond_0

    return-void

    .line 130
    :cond_0
    const/16 p2, 0xa

    const/16 v0, 0x190

    if-ne p1, p2, :cond_1

    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->pendingCameraUri:Landroid/net/Uri;

    if-eqz p2, :cond_1

    .line 131
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->pendingCameraUri:Landroid/net/Uri;

    invoke-virtual {p2}, Landroid/net/Uri;->toString()Ljava/lang/String;

    move-result-object p2

    iput-object p2, p1, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    .line 132
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->imgPreview:Landroid/widget/ImageView;

    invoke-static {p0, p1, p2, v0}, Lcom/dragonis/checklist/ImageUtil;->loadThumbnail(Landroid/content/Context;Ljava/lang/String;Landroid/widget/ImageView;I)V

    goto :goto_1

    .line 133
    :cond_1
    const/16 p2, 0xb

    if-ne p1, p2, :cond_2

    if-eqz p3, :cond_2

    invoke-virtual {p3}, Landroid/content/Intent;->getData()Landroid/net/Uri;

    move-result-object p1

    if-eqz p1, :cond_2

    .line 134
    invoke-virtual {p3}, Landroid/content/Intent;->getData()Landroid/net/Uri;

    move-result-object p1

    .line 136
    :try_start_0
    invoke-virtual {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->getContentResolver()Landroid/content/ContentResolver;

    move-result-object p2

    const/4 p3, 0x1

    invoke-virtual {p2, p1, p3}, Landroid/content/ContentResolver;->takePersistableUriPermission(Landroid/net/Uri;I)V
    :try_end_0
    .catch Ljava/lang/SecurityException; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    .line 137
    :catch_0
    move-exception p2

    :goto_0
    nop

    .line 138
    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    invoke-virtual {p1}, Landroid/net/Uri;->toString()Ljava/lang/String;

    move-result-object p1

    iput-object p1, p2, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    .line 139
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    iget-object p2, p0, Lcom/dragonis/checklist/AddEditItemActivity;->imgPreview:Landroid/widget/ImageView;

    invoke-static {p0, p1, p2, v0}, Lcom/dragonis/checklist/ImageUtil;->loadThumbnail(Landroid/content/Context;Ljava/lang/String;Landroid/widget/ImageView;I)V

    .line 141
    :cond_2
    :goto_1
    return-void
.end method

.method protected onCreate(Landroid/os/Bundle;)V
    .locals 3

    .line 43
    invoke-super {p0, p1}, Landroid/app/Activity;->onCreate(Landroid/os/Bundle;)V

    .line 44
    const/high16 p1, 0x7f040000

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->setContentView(I)V

    .line 45
    invoke-static {p0}, Lcom/dragonis/checklist/Store;->get(Landroid/content/Context;)Lcom/dragonis/checklist/Store;

    move-result-object p1

    iput-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->store:Lcom/dragonis/checklist/Store;

    .line 47
    invoke-virtual {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->getIntent()Landroid/content/Intent;

    move-result-object p1

    const-string v0, "item_id"

    invoke-virtual {p1, v0}, Landroid/content/Intent;->getStringExtra(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    .line 48
    if-eqz p1, :cond_0

    .line 49
    iget-object v0, p0, Lcom/dragonis/checklist/AddEditItemActivity;->store:Lcom/dragonis/checklist/Store;

    invoke-virtual {v0, p1}, Lcom/dragonis/checklist/Store;->findById(Ljava/lang/String;)Lcom/dragonis/checklist/ChecklistItem;

    move-result-object p1

    iput-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    .line 51
    :cond_0
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    const/4 v0, 0x0

    if-nez p1, :cond_1

    const/4 p1, 0x1

    goto :goto_0

    :cond_1
    const/4 p1, 0x0

    :goto_0
    iput-boolean p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->isNew:Z

    .line 52
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    if-nez p1, :cond_2

    .line 53
    new-instance p1, Lcom/dragonis/checklist/ChecklistItem;

    invoke-direct {p1}, Lcom/dragonis/checklist/ChecklistItem;-><init>()V

    iput-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    .line 56
    :cond_2
    const p1, 0x7f030012

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/ImageView;

    iput-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->imgPreview:Landroid/widget/ImageView;

    .line 57
    const p1, 0x7f03000e

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/EditText;

    iput-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->editName:Landroid/widget/EditText;

    .line 58
    const p1, 0x7f03000b

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/LinearLayout;

    iput-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->containerChecklist:Landroid/widget/LinearLayout;

    .line 59
    const p1, 0x7f030003

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/TextView;

    iput-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->btnDelete:Landroid/widget/TextView;

    .line 60
    const p1, 0x7f03001a

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/TextView;

    .line 61
    iget-boolean v1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->isNew:Z

    if-eqz v1, :cond_3

    const-string v1, "Nouvel \u00e9l\u00e9ment"

    goto :goto_1

    :cond_3
    const-string v1, "Modifier l\'\u00e9l\u00e9ment"

    :goto_1
    invoke-virtual {p1, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 63
    const p1, 0x7f030001

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    new-instance v1, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda3;

    invoke-direct {v1, p0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda3;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;)V

    invoke-virtual {p1, v1}, Landroid/view/View;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 64
    const p1, 0x7f030005

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    new-instance v1, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda4;

    invoke-direct {v1, p0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda4;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;)V

    invoke-virtual {p1, v1}, Landroid/view/View;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 65
    const/high16 p1, 0x7f030000

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    new-instance v1, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda5;

    invoke-direct {v1, p0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda5;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;)V

    invoke-virtual {p1, v1}, Landroid/view/View;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 66
    const p1, 0x7f030007

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    new-instance v1, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda6;

    invoke-direct {v1, p0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda6;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;)V

    invoke-virtual {p1, v1}, Landroid/view/View;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 68
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->editName:Landroid/widget/EditText;

    iget-object v1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object v1, v1, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    invoke-virtual {p1, v1}, Landroid/widget/EditText;->setText(Ljava/lang/CharSequence;)V

    .line 69
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    if-eqz p1, :cond_4

    .line 70
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    iget-object v1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->imgPreview:Landroid/widget/ImageView;

    const/16 v2, 0x190

    invoke-static {p0, p1, v1, v2}, Lcom/dragonis/checklist/ImageUtil;->loadThumbnail(Landroid/content/Context;Ljava/lang/String;Landroid/widget/ImageView;I)V

    .line 72
    :cond_4
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    if-eqz p1, :cond_5

    .line 73
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->item:Lcom/dragonis/checklist/ChecklistItem;

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-interface {p1}, Ljava/util/List;->iterator()Ljava/util/Iterator;

    move-result-object p1

    :goto_2
    invoke-interface {p1}, Ljava/util/Iterator;->hasNext()Z

    move-result v1

    if-eqz v1, :cond_5

    invoke-interface {p1}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v1

    check-cast v1, Ljava/lang/String;

    invoke-direct {p0, v1}, Lcom/dragonis/checklist/AddEditItemActivity;->addChecklistRow(Ljava/lang/String;)V

    goto :goto_2

    .line 75
    :cond_5
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->rowViews:Ljava/util/List;

    invoke-interface {p1}, Ljava/util/List;->isEmpty()Z

    move-result p1

    if-eqz p1, :cond_6

    const-string p1, ""

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/AddEditItemActivity;->addChecklistRow(Ljava/lang/String;)V

    .line 77
    :cond_6
    iget-boolean p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->isNew:Z

    if-nez p1, :cond_7

    .line 78
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->btnDelete:Landroid/widget/TextView;

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setVisibility(I)V

    .line 79
    iget-object p1, p0, Lcom/dragonis/checklist/AddEditItemActivity;->btnDelete:Landroid/widget/TextView;

    new-instance v0, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda7;

    invoke-direct {v0, p0}, Lcom/dragonis/checklist/AddEditItemActivity$$ExternalSyntheticLambda7;-><init>(Lcom/dragonis/checklist/AddEditItemActivity;)V

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    .line 81
    :cond_7
    return-void
.end method

.method public onRequestPermissionsResult(I[Ljava/lang/String;[I)V
    .locals 0

    .line 118
    invoke-super {p0, p1, p2, p3}, Landroid/app/Activity;->onRequestPermissionsResult(I[Ljava/lang/String;[I)V

    .line 119
    const/16 p2, 0x14

    if-ne p1, p2, :cond_0

    array-length p1, p3

    if-lez p1, :cond_0

    const/4 p1, 0x0

    aget p1, p3, p1

    if-nez p1, :cond_0

    .line 121
    invoke-direct {p0}, Lcom/dragonis/checklist/AddEditItemActivity;->launchCamera()V

    .line 123
    :cond_0
    return-void
.end method
