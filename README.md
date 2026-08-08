# DNPM Validator

Application to validate a data set in DNPM Datenmodell 2.1 and SE:dip data model format

## Usage

```
Usage: dnpm-validator [OPTIONS] <FILE>

Arguments:
  <FILE>  The file to be checked

Options:
      --type <SCHEMA>  The schema to be used [default: mtb] [possible values: mtb, rd]
  -h, --help           Print help
  -V, --version        Print version
```

![](image.png)

## Implemented validations

| Validation                           | MTB | RD |
|--------------------------------------|-----|----|
| JSON-Schema                          | ☑  | ☑ |
| Patient references                   | ☑  | ☑ |
| Recommendation references            | ☑  | ☑ |
| Claim references                     | ☑  | ☑ |
| Therapy references                   | ☑  | ☑ |
| Medication recommendation references | ☑  | -  |
| Therapy recommendation references    | -   | ☑ |

JSON-Schema validation is based on a more common JSON-Schema format than the one provided by DNPM:DIP. Any occurrences
of type references like `#Reference` have been replaced with e.g. `#/$defs/Reference` to make the validation work.