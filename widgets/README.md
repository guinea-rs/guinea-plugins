![guinea-widgets](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-widgets.png)

Widgets for guinea applications: tables, charts, drag-resize handles.

Not a plugin - nothing here installs into an application, it is a set of
components a page holds as fields and draws. It lives beside the plugins
because it varies on the same axis they do: the backend is a feature, and the
parts that are pure layout or geometry stay available without one.

| Module | What it has |
|---|---|
| `table` | A table whose columns sort, resize and reorder - reported to the page, which keeps the widths; `SortState` without a backend |
| `chart` | A line chart, live or still, drawn with Direct2D; `RingSeries` and the geometry without a backend |
| `chart::scatter` | Points in time against a value, on a linear or log scale, with named bands past its ends; hover, brush and click |
| `painted` | A view drawn with Direct2D that the page keeps as a field, with the pointer handed back |
| `resize` | The drag handle that resizes a column |

A scatter reads the page's own data where the page keeps it - anything that
implements `ScatterData` behind an `Rc` - and keeps no copy: handing the same
`Rc` again is no change, and data that knows its points' order can answer
`points_between` by search, so hovering and drawing ask only for the points
near the pointer and on screen.

```rust
scatter.publish(Rc::clone(&self.points), ScatterOptions { x: (from, to), ..ScatterOptions::default() });
```

## Features

| Feature | |
|---|---|
| `winui` (default) | The WinUI views, on `windows-reactor-pre` and `windows-canvas-pre` |

Without it, what is left is what needs no backend: sort state, ring series,
the charts' geometry.

`cargo run -p guinea-widgets --example scatter` shows a scatter chart.
