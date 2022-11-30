import as3
import json
import yaml
data = json.dumps({"dilec": "ciao"})
validator = yaml.safe_load(
    '''
Root:
  +type: Object
  dilec: Integer
''')


as3.verify(data, str(validator))
