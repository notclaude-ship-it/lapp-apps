.class public final synthetic Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda1;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# instance fields
.field public final synthetic f$0:Lcom/dragonis/checklist/MainActivity;

.field public final synthetic f$1:Lcom/dragonis/checklist/ChecklistItem;


# direct methods
.method public synthetic constructor <init>(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/ChecklistItem;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda1;->f$0:Lcom/dragonis/checklist/MainActivity;

    iput-object p2, p0, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda1;->f$1:Lcom/dragonis/checklist/ChecklistItem;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 2

    .line 0
    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda1;->f$0:Lcom/dragonis/checklist/MainActivity;

    iget-object v1, p0, Lcom/dragonis/checklist/MainActivity$$ExternalSyntheticLambda1;->f$1:Lcom/dragonis/checklist/ChecklistItem;

    invoke-static {v0, v1, p1, p2}, Lcom/dragonis/checklist/MainActivity;->$r8$lambda$mQVyOIbOomEeL2sP1gQENKfCqDk(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/ChecklistItem;Landroid/content/DialogInterface;I)V

    return-void
.end method
