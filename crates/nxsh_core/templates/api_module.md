# Module: {{module_name}}

{{description}}

## Functions

{{#functions}}
### {{name}}

```rust
{{signature}}
```

{{description}}

{{#parameters}}
- **{{name}}** ({{param_type}}): {{description}}
{{/parameters}}

{{#return_type}}
**Returns**: {{return_type}}
{{/return_type}}

{{#examples}}
#### Example: {{title}}

```rust
{{code}}
```

{{description}}

{{#expected_output}}
Expected output:
```
{{expected_output}}
```
{{/expected_output}}
{{/examples}}

{{/functions}}

## Types

{{#types}}
### {{name}} ({{type_kind}})

{{description}}

{{#fields}}
- **{{name}}** ({{field_type}}): {{description}}
{{/fields}}

{{#methods}}
#### {{name}}

```rust
{{signature}}
```

{{description}}
{{/methods}}

{{/types}}

## Constants

{{#constants}}
### {{name}}

```rust
{{name}}: {{type_info}} = {{value}};
```

{{description}}
{{/constants}}
