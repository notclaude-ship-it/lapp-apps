package com.dragonis.checklist;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.DialogInterface;
import android.content.Intent;
import android.graphics.ColorFilter;
import android.os.Bundle;
import android.util.TypedValue;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.AdapterView;
import android.widget.BaseAdapter;
import android.widget.GridView;
import android.widget.ImageView;
import android.widget.ListAdapter;
import android.widget.TextView;
import java.util.List;

/* loaded from: classes.dex */
public class MainActivity extends Activity {
    private Adapter adapter;
    private GridView grid;
    private Store store;

    @Override // android.app.Activity
    protected void onCreate(Bundle bundle) {
        super.onCreate(bundle);
        setContentView(R.layout.activity_main);
        this.store = Store.get(this);
        this.grid = (GridView) findViewById(R.id.grid_items);
        this.adapter = new Adapter();
        this.grid.setAdapter((ListAdapter) this.adapter);
        this.grid.setOnItemClickListener(new AdapterView.OnItemClickListener() { // from class: com.dragonis.checklist.MainActivity$$ExternalSyntheticLambda2
            @Override // android.widget.AdapterView.OnItemClickListener
            public final void onItemClick(AdapterView adapterView, View view, int i, long j) {
                MainActivity.this.lambda$onCreate$0(adapterView, view, i, j);
            }
        });
        this.grid.setOnItemLongClickListener(new AdapterView.OnItemLongClickListener() { // from class: com.dragonis.checklist.MainActivity$$ExternalSyntheticLambda3
            @Override // android.widget.AdapterView.OnItemLongClickListener
            public final boolean onItemLongClick(AdapterView adapterView, View view, int i, long j) {
                boolean lambda$onCreate$1;
                lambda$onCreate$1 = MainActivity.this.lambda$onCreate$1(adapterView, view, i, j);
                return lambda$onCreate$1;
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCreate$0(AdapterView adapterView, View view, int i, long j) {
        onCellClick(i);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ boolean lambda$onCreate$1(AdapterView adapterView, View view, int i, long j) {
        onCellLongClick(i);
        return true;
    }

    @Override // android.app.Activity
    protected void onResume() {
        super.onResume();
        this.adapter.notifyDataSetChanged();
    }

    private void onCellClick(int i) {
        List<ChecklistItem> items = this.store.getItems();
        if (i == items.size()) {
            startActivity(new Intent(this, (Class<?>) AddEditItemActivity.class));
            return;
        }
        ChecklistItem checklistItem = items.get(i);
        checklistItem.purgeStaleProgress();
        this.store.save();
        if (!checklistItem.isConfigured()) {
            Intent intent = new Intent(this, (Class<?>) AddEditItemActivity.class);
            intent.putExtra("item_id", checklistItem.id);
            startActivity(intent);
        } else {
            Intent intent2 = new Intent(this, (Class<?>) SessionActivity.class);
            intent2.putExtra("item_id", checklistItem.id);
            startActivity(intent2);
        }
    }

    private void onCellLongClick(int i) {
        List<ChecklistItem> items = this.store.getItems();
        if (i >= items.size()) {
            return;
        }
        final ChecklistItem checklistItem = items.get(i);
        new AlertDialog.Builder(this).setTitle(checklistItem.name).setItems(new CharSequence[]{"Modifier", "Supprimer"}, new DialogInterface.OnClickListener() { // from class: com.dragonis.checklist.MainActivity$$ExternalSyntheticLambda1
            @Override // android.content.DialogInterface.OnClickListener
            public final void onClick(DialogInterface dialogInterface, int i2) {
                MainActivity.this.lambda$onCellLongClick$3(checklistItem, dialogInterface, i2);
            }
        }).show();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCellLongClick$3(final ChecklistItem checklistItem, DialogInterface dialogInterface, int i) {
        if (i == 0) {
            Intent intent = new Intent(this, (Class<?>) AddEditItemActivity.class);
            intent.putExtra("item_id", checklistItem.id);
            startActivity(intent);
        } else {
            new AlertDialog.Builder(this).setTitle("Supprimer « " + checklistItem.name + " » ?").setMessage("La check-list et la progression du jour seront supprimées. L'historique des photos passées reste sur l'appareil.").setPositiveButton("Supprimer", new DialogInterface.OnClickListener() { // from class: com.dragonis.checklist.MainActivity$$ExternalSyntheticLambda0
                @Override // android.content.DialogInterface.OnClickListener
                public final void onClick(DialogInterface dialogInterface2, int i2) {
                    MainActivity.this.lambda$onCellLongClick$2(checklistItem, dialogInterface2, i2);
                }
            }).setNegativeButton("Annuler", (DialogInterface.OnClickListener) null).show();
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public /* synthetic */ void lambda$onCellLongClick$2(ChecklistItem checklistItem, DialogInterface dialogInterface, int i) {
        this.store.removeItem(checklistItem.id);
        this.adapter.notifyDataSetChanged();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public int dp(int i) {
        return (int) TypedValue.applyDimension(1, i, getResources().getDisplayMetrics());
    }

    /* JADX INFO: Access modifiers changed from: private */
    /* loaded from: classes.dex */
    public class Adapter extends BaseAdapter {
        private Adapter() {
        }

        @Override // android.widget.Adapter
        public int getCount() {
            return MainActivity.this.store.getItems().size() + 1;
        }

        @Override // android.widget.Adapter
        public Object getItem(int i) {
            return Integer.valueOf(i);
        }

        @Override // android.widget.Adapter
        public long getItemId(int i) {
            return i;
        }

        @Override // android.widget.Adapter
        public View getView(int i, View view, ViewGroup viewGroup) {
            if (view == null) {
                view = LayoutInflater.from(MainActivity.this).inflate(R.layout.grid_item, viewGroup, false);
            }
            View findViewById = view.findViewById(R.id.card_root);
            ImageView imageView = (ImageView) view.findViewById(R.id.img_icon);
            TextView textView = (TextView) view.findViewById(R.id.txt_name);
            TextView textView2 = (TextView) view.findViewById(R.id.txt_badge);
            textView2.setVisibility(8);
            List<ChecklistItem> items = MainActivity.this.store.getItems();
            if (i == items.size()) {
                findViewById.setBackgroundResource(R.drawable.bg_add_tile);
                imageView.setImageDrawable(null);
                imageView.setBackgroundResource(0);
                imageView.setImageResource(R.drawable.ic_add);
                imageView.setPadding(MainActivity.this.dp(40), MainActivity.this.dp(40), MainActivity.this.dp(40), MainActivity.this.dp(40));
                imageView.setColorFilter(MainActivity.this.getColor(R.color.md_primary));
                textView.setText("Ajouter");
                textView.setTextColor(MainActivity.this.getColor(R.color.md_primary));
            } else {
                ChecklistItem checklistItem = items.get(i);
                findViewById.setBackgroundResource(R.drawable.bg_card);
                imageView.setPadding(0, 0, 0, 0);
                imageView.setColorFilter((ColorFilter) null);
                imageView.setBackgroundResource(R.drawable.bg_thumb);
                if (checklistItem.iconUri != null) {
                    ImageUtil.loadThumbnail(MainActivity.this, checklistItem.iconUri, imageView, MainActivity.this.dp(130));
                } else {
                    imageView.setImageResource(R.drawable.ic_image);
                    imageView.setPadding(MainActivity.this.dp(40), MainActivity.this.dp(40), MainActivity.this.dp(40), MainActivity.this.dp(40));
                    imageView.setColorFilter(MainActivity.this.getColor(R.color.md_on_surface_variant));
                }
                textView.setText(checklistItem.name);
                textView.setTextColor(MainActivity.this.getColor(R.color.md_on_surface));
                checklistItem.purgeStaleProgress();
                if (!checklistItem.isConfigured()) {
                    textView2.setText("À configurer");
                    textView2.setVisibility(0);
                } else if (checklistItem.progress != null) {
                    textView2.setText("En cours");
                    textView2.setVisibility(0);
                }
            }
            return view;
        }
    }
}
