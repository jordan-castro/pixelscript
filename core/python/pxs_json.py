import json


def encode(object):
    return json.dumps(object)


def decode(string):
    return json.loads(string)