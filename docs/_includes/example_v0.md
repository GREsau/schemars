{% capture input %}examples/{{ include.name }}.rs{% endcapture %}
{% capture output %}examples/{{ include.name }}.schema.json{% endcapture %}

```rust
{% include {{ input }} %}
```

<details>
<summary>Click to see the output JSON schema...</summary>

```json
{% include {{ output }} -%}
```
</details>
