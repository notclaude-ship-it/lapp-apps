.class public Lcom/dragonis/checklist/ChecklistItem;
.super Ljava/lang/Object;
.source "ChecklistItem.java"


# instance fields
.field public checklist:Ljava/util/List;
    .annotation system Ldalvik/annotation/Signature;
        value = {
            "Ljava/util/List<",
            "Ljava/lang/String;",
            ">;"
        }
    .end annotation
.end field

.field public history:Ljava/util/List;
    .annotation system Ldalvik/annotation/Signature;
        value = {
            "Ljava/util/List<",
            "Lcom/dragonis/checklist/Session;",
            ">;"
        }
    .end annotation
.end field

.field public iconUri:Ljava/lang/String;

.field public id:Ljava/lang/String;

.field public name:Ljava/lang/String;

.field public progress:Lcom/dragonis/checklist/Progress;


# direct methods
.method public constructor <init>()V
    .locals 1

    .line 19
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    .line 15
    new-instance v0, Ljava/util/ArrayList;

    invoke-direct {v0}, Ljava/util/ArrayList;-><init>()V

    iput-object v0, p0, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    .line 17
    new-instance v0, Ljava/util/ArrayList;

    invoke-direct {v0}, Ljava/util/ArrayList;-><init>()V

    iput-object v0, p0, Lcom/dragonis/checklist/ChecklistItem;->history:Ljava/util/List;

    .line 20
    invoke-static {}, Ljava/util/UUID;->randomUUID()Ljava/util/UUID;

    move-result-object v0

    invoke-virtual {v0}, Ljava/util/UUID;->toString()Ljava/lang/String;

    move-result-object v0

    iput-object v0, p0, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    .line 21
    return-void
.end method

.method public static fromJson(Lorg/json/JSONObject;)Lcom/dragonis/checklist/ChecklistItem;
    .locals 7
    .annotation system Ldalvik/annotation/Throws;
        value = {
            Lorg/json/JSONException;
        }
    .end annotation

    .line 51
    new-instance v0, Lcom/dragonis/checklist/ChecklistItem;

    invoke-direct {v0}, Lcom/dragonis/checklist/ChecklistItem;-><init>()V

    .line 52
    invoke-static {}, Ljava/util/UUID;->randomUUID()Ljava/util/UUID;

    move-result-object v1

    invoke-virtual {v1}, Ljava/util/UUID;->toString()Ljava/lang/String;

    move-result-object v1

    const-string v2, "id"

    invoke-virtual {p0, v2, v1}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    .line 53
    const-string v1, "name"

    const-string v2, ""

    invoke-virtual {p0, v1, v2}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    .line 54
    const-string v1, "iconUri"

    invoke-virtual {p0, v1}, Lorg/json/JSONObject;->isNull(Ljava/lang/String;)Z

    move-result v2

    const/4 v3, 0x0

    if-eqz v2, :cond_0

    move-object v1, v3

    goto :goto_0

    :cond_0
    invoke-virtual {p0, v1, v3}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    :goto_0
    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    .line 55
    new-instance v1, Ljava/util/ArrayList;

    invoke-direct {v1}, Ljava/util/ArrayList;-><init>()V

    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    .line 56
    const-string v1, "checklist"

    invoke-virtual {p0, v1}, Lorg/json/JSONObject;->optJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object v1

    .line 57
    const/4 v2, 0x0

    if-eqz v1, :cond_1

    const/4 v4, 0x0

    :goto_1
    invoke-virtual {v1}, Lorg/json/JSONArray;->length()I

    move-result v5

    if-ge v4, v5, :cond_1

    iget-object v5, v0, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-virtual {v1, v4}, Lorg/json/JSONArray;->optString(I)Ljava/lang/String;

    move-result-object v6

    invoke-interface {v5, v6}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    add-int/lit8 v4, v4, 0x1

    goto :goto_1

    .line 58
    :cond_1
    const-string v1, "progress"

    invoke-virtual {p0, v1}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v1

    .line 59
    if-eqz v1, :cond_2

    invoke-static {v1}, Lcom/dragonis/checklist/Progress;->fromJson(Lorg/json/JSONObject;)Lcom/dragonis/checklist/Progress;

    move-result-object v3

    :cond_2
    iput-object v3, v0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    .line 60
    new-instance v1, Ljava/util/ArrayList;

    invoke-direct {v1}, Ljava/util/ArrayList;-><init>()V

    iput-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->history:Ljava/util/List;

    .line 61
    const-string v1, "history"

    invoke-virtual {p0, v1}, Lorg/json/JSONObject;->optJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object p0

    .line 62
    if-eqz p0, :cond_3

    .line 63
    nop

    :goto_2
    invoke-virtual {p0}, Lorg/json/JSONArray;->length()I

    move-result v1

    if-ge v2, v1, :cond_3

    .line 64
    iget-object v1, v0, Lcom/dragonis/checklist/ChecklistItem;->history:Ljava/util/List;

    invoke-virtual {p0, v2}, Lorg/json/JSONArray;->getJSONObject(I)Lorg/json/JSONObject;

    move-result-object v3

    invoke-static {v3}, Lcom/dragonis/checklist/Session;->fromJson(Lorg/json/JSONObject;)Lcom/dragonis/checklist/Session;

    move-result-object v3

    invoke-interface {v1, v3}, Ljava/util/List;->add(Ljava/lang/Object;)Z

    .line 63
    add-int/lit8 v2, v2, 0x1

    goto :goto_2

    .line 67
    :cond_3
    return-object v0
.end method


# virtual methods
.method public isConfigured()Z
    .locals 1

    .line 24
    iget-object v0, p0, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    if-eqz v0, :cond_0

    iget-object v0, p0, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-interface {v0}, Ljava/util/List;->isEmpty()Z

    move-result v0

    if-nez v0, :cond_0

    const/4 v0, 0x1

    goto :goto_0

    :cond_0
    const/4 v0, 0x0

    :goto_0
    return v0
.end method

.method public purgeStaleProgress()V
    .locals 2

    .line 32
    iget-object v0, p0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    if-eqz v0, :cond_0

    invoke-static {}, Lcom/dragonis/checklist/DateUtil;->today()Ljava/lang/String;

    move-result-object v0

    iget-object v1, p0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    iget-object v1, v1, Lcom/dragonis/checklist/Progress;->date:Ljava/lang/String;

    invoke-virtual {v0, v1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-nez v0, :cond_0

    .line 33
    const/4 v0, 0x0

    iput-object v0, p0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    .line 35
    :cond_0
    return-void
.end method

.method public toJson()Lorg/json/JSONObject;
    .locals 4
    .annotation system Ldalvik/annotation/Throws;
        value = {
            Lorg/json/JSONException;
        }
    .end annotation

    .line 38
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {v0}, Lorg/json/JSONObject;-><init>()V

    .line 39
    const-string v1, "id"

    iget-object v2, p0, Lcom/dragonis/checklist/ChecklistItem;->id:Ljava/lang/String;

    invoke-virtual {v0, v1, v2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    .line 40
    const-string v1, "name"

    iget-object v2, p0, Lcom/dragonis/checklist/ChecklistItem;->name:Ljava/lang/String;

    invoke-virtual {v0, v1, v2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    .line 41
    iget-object v1, p0, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    if-eqz v1, :cond_0

    const-string v1, "iconUri"

    iget-object v2, p0, Lcom/dragonis/checklist/ChecklistItem;->iconUri:Ljava/lang/String;

    invoke-virtual {v0, v1, v2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    .line 42
    :cond_0
    new-instance v1, Lorg/json/JSONArray;

    iget-object v2, p0, Lcom/dragonis/checklist/ChecklistItem;->checklist:Ljava/util/List;

    invoke-direct {v1, v2}, Lorg/json/JSONArray;-><init>(Ljava/util/Collection;)V

    const-string v2, "checklist"

    invoke-virtual {v0, v2, v1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    .line 43
    iget-object v1, p0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    if-eqz v1, :cond_1

    iget-object v1, p0, Lcom/dragonis/checklist/ChecklistItem;->progress:Lcom/dragonis/checklist/Progress;

    invoke-virtual {v1}, Lcom/dragonis/checklist/Progress;->toJson()Lorg/json/JSONObject;

    move-result-object v1

    const-string v2, "progress"

    invoke-virtual {v0, v2, v1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    .line 44
    :cond_1
    new-instance v1, Lorg/json/JSONArray;

    invoke-direct {v1}, Lorg/json/JSONArray;-><init>()V

    .line 45
    iget-object v2, p0, Lcom/dragonis/checklist/ChecklistItem;->history:Ljava/util/List;

    invoke-interface {v2}, Ljava/util/List;->iterator()Ljava/util/Iterator;

    move-result-object v2

    :goto_0
    invoke-interface {v2}, Ljava/util/Iterator;->hasNext()Z

    move-result v3

    if-eqz v3, :cond_2

    invoke-interface {v2}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v3

    check-cast v3, Lcom/dragonis/checklist/Session;

    invoke-virtual {v3}, Lcom/dragonis/checklist/Session;->toJson()Lorg/json/JSONObject;

    move-result-object v3

    invoke-virtual {v1, v3}, Lorg/json/JSONArray;->put(Ljava/lang/Object;)Lorg/json/JSONArray;

    goto :goto_0

    .line 46
    :cond_2
    const-string v2, "history"

    invoke-virtual {v0, v2, v1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    .line 47
    return-object v0
.end method
