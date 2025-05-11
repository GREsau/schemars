{% capture input %}examples_v0/{{ include.name }}.rs{% endcapture %}
{% capture output %}examples_v0/{{ include.name }}.schema.json{% endcapture %}

```rust
{% include {{ input }} %}
```

<details>
<summary>Click to see the output JSON schema...</summary>

```json
{% include {{ output }} -%}
```

</details>
