from yoyo import print
from yoyo import net


response = net.client.get("https://jsonplaceholder.typicode.com/todos/1")
print(response)