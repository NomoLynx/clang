# CLang type conversion mapping

this mapping define the c language type conversion table. the type0 is the from type and type1 is the to type. the conversion function template name is the snippet name which is stored in the code_snippet folder's conversion file. in short, type0 to type1

| Type0 | Type1 | Conversion Function Template Name |
| --- | --- | --- |
| double | float | convert_to_f64 |
| float | double | convert_to_f32 |
