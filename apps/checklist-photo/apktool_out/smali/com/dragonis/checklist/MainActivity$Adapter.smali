.class Lcom/dragonis/checklist/MainActivity$Adapter;
.super Landroid/widget/BaseAdapter;
.source "MainActivity.java"


# annotations
.annotation system Ldalvik/annotation/EnclosingClass;
    value = Lcom/dragonis/checklist/MainActivity;
.end annotation

.annotation system Ldalvik/annotation/InnerClass;
    accessFlags = 0x2
    name = "Adapter"
.end annotation


# instance fields
.field final synthetic this$0:Lcom/dragonis/checklist/MainActivity;


# direct methods
.method private constructor <init>(Lcom/dragonis/checklist/MainActivity;)V
    .locals 0
    .annotation system Ldalvik/annotation/MethodParameters;
        accessFlags = {
            0x1010
        }
        names = {
            null
        }
    .end annotation

    .line 99
    iput-object p1, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-direct {p0}, Landroid/widget/BaseAdapter;-><init>()V

    return-void
.end method

.method synthetic constructor <init>(Lcom/dragonis/checklist/MainActivity;Lcom/dragonis/checklist/MainActivity-IA;)V
    .locals 0

    invoke-direct {p0, p1}, Lcom/dragonis/checklist/MainActivity$Adapter;-><init>(Lcom/dragonis/checklist/MainActivity;)V

    return-void
.end method


# virtual methods
.method public getCount()I
    .locals 1

    .line 102
    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {v0}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$fgetstore(Lcom/dragonis/checklist/MainActivity;)Lcom/dragonis/checklist/Store;

    move-result-object v0

    invoke-virtual {v0}, Lcom/dragonis/checklist/Store;->getItems()Ljava/util/List;

    move-result-object v0

    invoke-interface {v0}, Ljava/util/List;->size()I

    move-result v0

    add-int/lit8 v0, v0, 0x1

    return v0
.end method

.method public getItem(I)Ljava/lang/Object;
    .locals 0

    .line 107
    invoke-static {p1}, Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;

    move-result-object p1

    return-object p1
.end method

.method public getItemId(I)J
    .locals 2

    .line 112
    int-to-long v0, p1

    return-wide v0
.end method

.method public getView(ILandroid/view/View;Landroid/view/ViewGroup;)Landroid/view/View;
    .locals 8

    .line 117
    nop

    .line 118
    const/4 v0, 0x0

    if-nez p2, :cond_0

    .line 119
    iget-object p2, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {p2}, Landroid/view/LayoutInflater;->from(Landroid/content/Context;)Landroid/view/LayoutInflater;

    move-result-object p2

    const v1, 0x7f040003

    invoke-virtual {p2, v1, p3, v0}, Landroid/view/LayoutInflater;->inflate(ILandroid/view/ViewGroup;Z)Landroid/view/View;

    move-result-object p2

    .line 121
    :cond_0
    const p3, 0x7f030009

    invoke-virtual {p2, p3}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object p3

    .line 122
    const v1, 0x7f030011

    invoke-virtual {p2, v1}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object v1

    check-cast v1, Landroid/widget/ImageView;

    .line 123
    const v2, 0x7f030016

    invoke-virtual {p2, v2}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object v2

    check-cast v2, Landroid/widget/TextView;

    .line 124
    const v3, 0x7f030015

    invoke-virtual {p2, v3}, Landroid/view/View;->findViewById(I)Landroid/view/View;

    move-result-object v3

    check-cast v3, Landroid/widget/TextView;

    .line 125
    const/16 v4, 0x8

    invoke-virtual {v3, v4}, Landroid/widget/TextView;->setVisibility(I)V

    .line 127
    iget-object v4, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {v4}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$fgetstore(Lcom/dragonis/checklist/MainActivity;)Lcom/dragonis/checklist/Store;

    move-result-object v4

    invoke-virtual {v4}, Lcom/dragonis/checklist/Store;->getItems()Ljava/util/List;

    move-result-object v4

    .line 128
    invoke-interface {v4}, Ljava/util/List;->size()I

    move-result v5

    const/4 v6, 0x0

    const/16 v7, 0x28

    if-ne p1, v5, :cond_1

    .line 129
    const/high16 p1, 0x7f020000

    invoke-virtual {p3, p1}, Landroid/view/View;->setBackgroundResource(I)V

    .line 130
    invoke-virtual {v1, v6}, Landroid/widget/ImageView;->setImageDrawable(Landroid/graphics/drawable/Drawable;)V

    .line 131
    invoke-virtual {v1, v0}, Landroid/widget/ImageView;->setBackgroundResource(I)V

    .line 132
    const p1, 0x7f020006

    invoke-virtual {v1, p1}, Landroid/widget/ImageView;->setImageResource(I)V

    .line 133
    iget-object p1, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {p1, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result p1

    iget-object p3, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {p3, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result p3

    iget-object v0, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {v0, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result v0

    iget-object v3, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {v3, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result v3

    invoke-virtual {v1, p1, p3, v0, v3}, Landroid/widget/ImageView;->setPadding(IIII)V

    .line 134
    iget-object p1, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    const p3, 0x7f01000e

    invoke-virtual {p1, p3}, Lcom/dragonis/checklist/MainActivity;->getColor(I)I

    move-result p1

    invoke-virtual {v1, p1}, Landroid/widget/ImageView;->setColorFilter(I)V

    .line 135
    const-string p1, "Ajouter"

    invoke-virtual {v2, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 136
    iget-object p1, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-virtual {p1, p3}, Lcom/dragonis/checklist/MainActivity;->getColor(I)I

    move-result p1

    invoke-virtual {v2, p1}, Landroid/widget/TextView;->setTextColor(I)V

    goto/16 :goto_1

    .line 138
    :cond_1
    invoke-interface {v4, p1}, Ljava/util/List;->get(I)Ljava/lang/Object;

    move-result-object p1

    check-cast p1, Lcom/dragonis/checklist/ChecklistItem;

    .line 139
    const v4, 0x7f020003

    invoke-virtual {p3, v4}, Landroid/view/View;->setBackgroundResource(I)V

    .line 140
    invoke-virtual {v1, v0, v0, v0, v0}, Landroid/widget/ImageView;->setPadding(IIII)V

    .line 141
    invoke-virtual {v1, v6}, Landroid/widget/ImageView;->setColorFilter(Landroid/graphics/ColorFilter;)V

    .line 142
    const p3, 0x7f020005

    invoke-virtual {v1, p3}, Landroid/widget/ImageView;->setBackgroundResource(I)V

    .line 143
    iget-object p3, p1, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    if-eqz p3, :cond_2

    .line 144
    iget-object p3, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    iget-object v4, p1, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    iget-object v5, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    const/16 v6, 0x82

    invoke-static {v5, v6}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result v5

    invoke-static {p3, v4, v1, v5}, Lcom/dragonis/checklist/ImageUtil;->loadThumbnail(Landroid/content/Context;Ljava/lang/String;Landroid/widget/ImageView;I)V

    goto :goto_0

    .line 146
    :cond_2
    const p3, 0x7f02000c

    invoke-virtual {v1, p3}, Landroid/widget/ImageView;->setImageResource(I)V

    .line 147
    iget-object p3, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {p3, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result p3

    iget-object v4, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {v4, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result v4

    iget-object v5, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {v5, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result v5

    iget-object v6, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    invoke-static {v6, v7}, Lcom/dragonis/checklist/MainActivity;->-$$Nest$mdp(Lcom/dragonis/checklist/MainActivity;I)I

    move-result v6

    invoke-virtual {v1, p3, v4, v5, v6}, Landroid/widget/ImageView;->setPadding(IIII)V

    .line 148
    iget-object p3, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    const v4, 0x7f01000a

    invoke-virtual {p3, v4}, Lcom/dragonis/checklist/MainActivity;->getColor(I)I

    move-result p3

    invoke-virtual {v1, p3}, Landroid/widget/ImageView;->setColorFilter(I)V

    .line 150
    :goto_0
    iget-object p3, p1, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    invoke-virtual {v2, p3}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 151
    iget-object p3, p0, Lcom/dragonis/checklist/MainActivity$Adapter;->this$0:Lcom/dragonis/checklist/MainActivity;

    const v1, 0x7f010009

    invoke-virtual {p3, v1}, Lcom/dragonis/checklist/MainActivity;->getColor(I)I

    move-result p3

    invoke-virtual {v2, p3}, Landroid/widget/TextView;->setTextColor(I)V

    .line 153
    invoke-virtual {p1}, Lcom/dragonis/checklist/ChecklistItem;->purgeStaleProgress()V

    .line 154
    invoke-virtual {p1}, Lcom/dragonis/checklist/ChecklistItem;->isConfigured()Z

    move-result p3

    if-nez p3, :cond_3

    .line 155
    const-string p1, "\u00c0 configurer"

    invoke-virtual {v3, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 156
    invoke-virtual {v3, v0}, Landroid/widget/TextView;->setVisibility(I)V

    goto :goto_1

    .line 157
    :cond_3
    iget-object p1, p1, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    if-eqz p1, :cond_4

    .line 158
    const-string p1, "En cours"

    invoke-virtual {v3, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 159
    invoke-virtual {v3, v0}, Landroid/widget/TextView;->setVisibility(I)V

    .line 162
    :cond_4
    :goto_1
    return-object p2
.end method
