.class public Lcom/dragonis/checklist/MainActivity;
.super Landroid/app/Activity;
.source "MainActivity.java"


# annotations
.annotation system Ldalvik/annotation/MemberClasses;
    value = {
        Lcom/dragonis/checklist/MainActivity$Adapter;
    }
.end annotation


# instance fields
.field private adapter:Lcom/dragonis/checklist/MainActivity$Adapter;

.field private grid:Landroid/widget/GridView;

.field private store:Lcom/dragonis/checklist/Store;


# direct methods
.method public static synthetic $r8$lambda$DOI6_NWhCPqUoLiewyJO3qasBy4(Lcom/dragonis/checklist/MainActivity;Landroid/widget/AdapterView;Landroid/view/View;IJ)V
    .locals 0

    invoke-direct/range {p0 .. p5}, Lcom/dragonis/checklist/MainActivity;->lambda$onCreate$0(Landroid/widget/AdapterView;Landroid/view/View;IJ)V

    return-void
.end method

.method public static synthetic $r8$lambda$Tq1Kb1I3jMztDKjo_0U2A13HFtU(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/ChecklistItem;Landroid/content/DialogInterface;I)V
    .locals 0

    invoke-direct {p0, p1, p2, p3}, Lcom/dragonis/checklist/MainActivity;->lambda$onCellLongClick$2(Lcom/dragonis/checklist/ChecklistItem;Landroid/content/DialogInterface;I)V

    return-void
.end method

.method public static synthetic $r8$lambda$dEw6lKiSWY4BXGsQqlyLcxoVwW8(Lcom/dragonis/checklist/MainActivity;Landroid/widget/AdapterView;Landroid/view/View;IJ)Z
    .locals 0

    invoke-direct/range {p0 .. p5}, Lcom/dragonis/checklist/MainActivity;->lambda$onCreate$1(Landroid/widget/AdapterView;Landroid/view/View;IJ)Z

    move-result p0

    return p0
.end method

.method public static synthetic $r8$lambda$mQVyOIbOomEeL2sP1gQENKfCqDk(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/ChecklistItem;Landroid/content/DialogInterface;I)V
    .locals 0

    invoke-direct {p0, p1, p2, p3}, Lcom/dragonis/checklist/MainActivity;->lambda$onCellLongClick$3(Lcom/dragonis/checklist/ChecklistItem;Landroid/content/DialogInterface;I)V

    return-void
.end method

.method static bridge synthetic -$$Nest$fgetstore(Lcom/dragonis/checklist/MainActivity;)Lcom/dragonis/checklist/Store;
    .locals 0

    iget-object p0, p0, Lcom/dragonis/checklist/MainActivity;->store:Lcom/dragonis/checklist/Store;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/MainActivity;->dp(I)I

    move-result p0

    return p0
.end method

.method public constructor <init>()V
    .locals 0

    .line 18
    invoke-direct {p0}, Landroid/app/Activity;-><init>()V

    return-void
.end method

.method private dp(I)I
    .locals 2

    .line 96
    int-to-float p1, p1

    invoke-virtual {p0}, Lcom/dragonis/checklist/MainActivity;->getResources()Landroid/content/res/Resources;

    move-result-object v0

    invoke-virtual {v0}, Landroid/content/res/Resources;->getDisplayMetrics()Landroid/util/DisplayMetrics;

    move-result-object v0

    const/4 v1, 0x1

    invoke-static {v1, p1, v0}, Landroid/util/TypedValue;->applyDimension(IFLandroid/util/DisplayMetrics;)F

    move-result p1

    float-to-int p1, p1

    return p1
.end method

.method private synthetic lambda$onCellLongClick$2(Lcom/dragonis/checklist/ChecklistItem;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 85
    iget-object p2, p0, Lcom/dragonis/checklist/MainActivity;->store:Lcom/dragonis/checklist/Store;

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    invoke-virtual {p2, p1}, Lcom/dragonis/checklist/Store;->removeItem(Ljava/lang/String;)V

    .line 86
    iget-object p1, p0, Lcom/dragonis/checklist/MainActivity;->adapter:Lcom/dragonis/checklist/MainActivity$Adapter;

    invoke-virtual {p1}, Lcom/dragonis/checklist/MainActivity$Adapter;->notifyDataSetChanged()V

    .line 87
    return-void
.end method

.method private synthetic lambda$onCellLongClick$3(Lcom/dragonis/checklist/ChecklistItem;Landroid/content/DialogInterface;I)V
    .locals 2

    .line 76
    if-nez p3, :cond_0

    .line 77
    new-instance p2, Landroid/content/Intent;

    const-class p3, Lcom/dragonis/checklist/AddEditItemActivity;

    invoke-direct {p2, p0, p3}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    .line 78
    const-string p3, "item_id"

    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    invoke-virtual {p2, p3, p1}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;

    .line 79
    invoke-virtual {p0, p2}, Lcom/dragonis/checklist/MainActivity;->startActivity(Landroid/content/Intent;)V

    .line 80
    goto :goto_0

    .line 81
    :cond_0
    new-instance p2, Landroid/app/AlertDialog$Builder;

    invoke-direct {p2, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    iget-object p3, p1, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    new-instance v0, Ljava/lang/StringBuilder;

    invoke-direct {v0}, Ljava/lang/StringBuilder;-><init>()V

    const-string v1, "Supprimer \u00ab "

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, p3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p3

    const-string v0, " \u00bb ?"

    invoke-virtual {p3, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p3

    invoke-virtual {p3}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p3

    .line 82
    invoke-virtual {p2, p3}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    .line 83
    const-string p3, "La check-list et la progression du jour seront supprim\u00e9es. L\'historique des photos pass\u00e9es reste sur l\'appareil."

    invoke-virtual {p2, p3}, Landroid/app/AlertDialog$Builder;->setMessage(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    new-instance p3, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda0;

    invoke-direct {p3, p0, p1}, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda0;-><init>(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/ChecklistItem;)V

    .line 84
    const-string p1, "Supprimer"

    invoke-virtual {p2, p1, p3}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 88
    const-string p2, "Annuler"

    const/4 p3, 0x0

    invoke-virtual {p1, p2, p3}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 89
    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 91
    :goto_0
    return-void
.end method

.method private synthetic lambda$onCreate$0(Landroid/widget/AdapterView;Landroid/view/View;IJ)V
    .locals 0

    .line 34
    invoke-direct {p0, p3}, Lcom/dragonis/checklist/MainActivity;->onCellClick(I)V

    return-void
.end method

.method private synthetic lambda$onCreate$1(Landroid/widget/AdapterView;Landroid/view/View;IJ)Z
    .locals 0

    .line 36
    invoke-direct {p0, p3}, Lcom/dragonis/checklist/MainActivity;->onCellLongClick(I)V

    .line 37
    const/4 p1, 0x1

    return p1
.end method

.method private onCellClick(I)V
    .locals 3

    .line 48
    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity;->store:Lcom/dragonis/checklist/Store;

    invoke-virtual {v0}, Lcom/dragonis/checklist/Store;->getItems()Ljava/util/List;

    move-result-object v0

    .line 49
    invoke-interface {v0}, Ljava/util/List;->size()I

    move-result v1

    if-ne p1, v1, :cond_0

    .line 51
    new-instance p1, Landroid/content/Intent;

    const-class v0, Lcom/dragonis/checklist/AddEditItemActivity;

    invoke-direct {p1, p0, v0}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/MainActivity;->startActivity(Landroid/content/Intent;)V

    .line 52
    return-void

    .line 54
    :cond_0
    invoke-interface {v0, p1}, Ljava/util/List;->get(I)Ljava/lang/Object;

    move-result-object p1

    check-cast p1, Lcom/dragonis/checklist/ChecklistItem;

    .line 55
    invoke-virtual {p1}, Lcom/dragonis/checklist/ChecklistItem;->purgeStaleProgress()V

    .line 56
    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity;->store:Lcom/dragonis/checklist/Store;

    invoke-virtual {v0}, Lcom/dragonis/checklist/Store;->save()V

    .line 57
    invoke-virtual {p1}, Lcom/dragonis/checklist/ChecklistItem;->isConfigured()Z

    move-result v0

    const-string v1, "item_id"

    if-nez v0, :cond_1

    .line 58
    new-instance v0, Landroid/content/Intent;

    const-class v2, Lcom/dragonis/checklist/AddEditItemActivity;

    invoke-direct {v0, p0, v2}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    .line 59
    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    invoke-virtual {v0, v1, p1}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;

    .line 60
    invoke-virtual {p0, v0}, Lcom/dragonis/checklist/MainActivity;->startActivity(Landroid/content/Intent;)V

    .line 61
    goto :goto_0

    .line 62
    :cond_1
    new-instance v0, Landroid/content/Intent;

    const-class v2, Lcom/dragonis/checklist/SessionActivity;

    invoke-direct {v0, p0, v2}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    .line 63
    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    invoke-virtual {v0, v1, p1}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;

    .line 64
    invoke-virtual {p0, v0}, Lcom/dragonis/checklist/MainActivity;->startActivity(Landroid/content/Intent;)V

    .line 66
    :goto_0
    return-void
.end method

.method private onCellLongClick(I)V
    .locals 4

    .line 69
    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity;->store:Lcom/dragonis/checklist/Store;

    invoke-virtual {v0}, Lcom/dragonis/checklist/Store;->getItems()Ljava/util/List;

    move-result-object v0

    .line 70
    invoke-interface {v0}, Ljava/util/List;->size()I

    move-result v1

    if-lt p1, v1, :cond_0

    return-void

    .line 71
    :cond_0
    invoke-interface {v0, p1}, Ljava/util/List;->get(I)Ljava/lang/Object;

    move-result-object p1

    check-cast p1, Lcom/dragonis/checklist/ChecklistItem;

    .line 73
    new-instance v0, Landroid/app/AlertDialog$Builder;

    invoke-direct {v0, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    iget-object v1, p1, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    .line 74
    invoke-virtual {v0, v1}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    const/4 v1, 0x2

    new-array v1, v1, [Ljava/lang/CharSequence;

    const/4 v2, 0x0

    const-string v3, "Modifier"

    aput-object v3, v1, v2

    const/4 v2, 0x1

    const-string v3, "Supprimer"

    aput-object v3, v1, v2

    new-instance v2, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda1;

    invoke-direct {v2, p0, p1}, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda1;-><init>(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/ChecklistItem;)V

    .line 75
    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 92
    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 93
    return-void
.end method


# virtual methods
.method protected onCreate(Landroid/os/Bundle;)V
    .locals 1

    .line 26
    invoke-super {p0, p1}, Landroid/app/Activity;->onCreate(Landroid/os/Bundle;)V

    .line 27
    const p1, 0x7f040001

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/MainActivity;->setContentView(I)V

    .line 29
    invoke-static {p0}, Lcom/dragonis/checklist/Store;->get(Landroid/content/Context;)Lcom/dragonis/checklist/Store;

    move-result-object p1

    iput-object p1, p0, Lcom/dragonis/checklist/MainActivity;->store:Lcom/dragonis/checklist/Store;

    .line 30
    const p1, 0x7f030010

    invoke-virtual {p0, p1}, Lcom/dragonis/checklist/MainActivity;->findViewById(I)Landroid/view/View;

    move-result-object p1

    check-cast p1, Landroid/widget/GridView;

    iput-object p1, p0, Lcom/dragonis/checklist/MainActivity;->grid:Landroid/widget/GridView;

    .line 31
    new-instance p1, Lcom/dragonis/checklist/MainActivity$Adapter;

    const/4 v0, 0x0

    invoke-direct {p1, p0, v0}, Lcom/dragonis/checklist/MainActivity$Adapter;-><init>(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/MainActivity-IA;)V

    iput-object p1, p0, Lcom/dragonis/checklist/MainActivity;->adapter:Lcom/dragonis/checklist/MainActivity$Adapter;

    .line 32
    iget-object p1, p0, Lcom/dragonis/checklist/MainActivity;->grid:Landroid/widget/GridView;

    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity;->adapter:Lcom/dragonis/checklist/MainActivity$Adapter;

    invoke-virtual {p1, v0}, Landroid/widget/GridView;->setAdapter(Landroid/widget/ListAdapter;)V

    .line 34
    iget-object p1, p0, Lcom/dragonis/checklist/MainActivity;->grid:Landroid/widget/GridView;

    new-instance v0, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda2;

    invoke-direct {v0, p0}, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda2;-><init>(Lcom/dragonis/checklist/MainActivity;)V

    invoke-virtual {p1, v0}, Landroid/widget/GridView;->setOnItemClickListener(Landroid/widget/AdapterView$OnItemClickListener;)V

    .line 35
    iget-object p1, p0, Lcom/dragonis/checklist/MainActivity;->grid:Landroid/widget/GridView;

    new-instance v0, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda3;

    invoke-direct {v0, p0}, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda3;-><init>(Lcom/dragonis/checklist/MainActivity;)V

    invoke-virtual {p1, v0}, Landroid/widget/GridView;->setOnItemLongClickListener(Landroid/widget/AdapterView$OnItemLongClickListener;)V

    .line 39
    return-void
.end method

.method protected onResume()V
    .locals 1

    .line 43
    invoke-super {p0}, Landroid/app/Activity;->onResume()V

    .line 44
    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity;->adapter:Lcom/dragonis/checklist/MainActivity$Adapter;

    invoke-virtual {v0}, Lcom/dragonis/checklist/MainActivity$Adapter;->notifyDataSetChanged()V

    .line 45
    return-void
.end method
