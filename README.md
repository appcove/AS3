| Json_type   |Required fields | Optional property |
|----------|------|------|
| `Object` |  - | -|
| `String` |   - | **max lenght** [*Integer*]: `MaxLength` ,`maxLength`, `max_length`<br> **min lenght** [*Integer*]: `MinLength` ,`minLength`, `min_length`  </br> **regex** [*String*]: `regex` |
| `Integer` |   - |**max** [*Integer*] : `max` </br> **min** [*Integer*]: `min`|
| `Map` | **key** [*String*, *Bool*, *Date*, *Integer*, *Double*] : `KeyType` </br> **value** [*Json_type*] : `ValueType`  | -|
| `List` | **value** [*Json_type*] : `ValueType` |- |



# General Exmaple
<table>
<tr>
<th>Input Data</th>
<th>Validator Config</th>
<th>Result</th>

</tr>
<tr>
<td>
<pre>
{
    "students": [
        {
        "surname": "Smith",
        "year": 2018,
        "grade": "B+"
        },
        {
        "surname": "Davis",
        "year": 2020,
        "grade": "A-"
        }
    ]
}
</pre>
</td>
<td>

```Yaml
Root:
  +type: Object
  students:
    +type: List
    +ValueType:
      +type: Object
      surname:
        +type: String
      year:
        +type: Integer
      grade:
        +type: String
```
</td>
<td>

```Text
✅
```
</td>

</tr>
</table>
