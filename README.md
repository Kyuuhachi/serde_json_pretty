# serde_json_pretty

[![crates.io](https://img.shields.io/crates/v/serde_json_pretty.svg)](https://crates.io/crates/serde_json_pretty)
[![docs.rs](https://docs.rs/serde_json_pretty/badge.svg)](https://docs.rs/serde_json_pretty)

<!-- cargo-rdme start -->

`serde_json` is great, but the formatting of the output... Not so much.
You get to choose between no whitespace whatsoever (good for interchange, but not human-readable),
or spaced-out with newlines between every single element (bad for interchange, and only barely human-readable).

This crate provides a middle ground: the overarching structure is formatted like `PrettyFormatter`,
but lists and objects consisting entirely of primitive values are formatted on a single line (but still not as densely as `CompactFormatter`).
The result looks something like this:

```json
{
 "INFO": {
   "name": "tremble_r1",
   "transform": [ 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -0.14999999, -0.099999994, -0.03, 1.0 ],
   "flags": [ 1074823168, 1081459343, 0 ]
 },
 "RTY2": { "material_variant": 0 },
 "LIG3": {
   "a": [ 1.0, 1.0, 1.0, 1.0 ],
   "b": 224,
   "c": 1.0,
   "d": [ 0.0, 45.0, 0.0, 0.0 ]
 },
 "INFZ": { "a": 11751, "b": 16629, "c": 11393, "d": 32769 },
 "JNTV": {
   "a": [ 0.0, 0.0, 0.0 ],
   "b": 2
 },
 "KAN7": "tremble_r1.KAN7",
 "PLU3": "tremble_r1.PLU3",
 "BBOX": {
   "min": [ -1.0, -1.0, 0.0 ],
   "max": [ 1.0, 1.0, 2.0 ],
   "radius": 0.0
 }
}
```

The space savings varies depending on the data, but some tests achieve around 70% reduction in lines.

<!-- cargo-rdme end -->
