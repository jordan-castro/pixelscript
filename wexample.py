class State: 
    def __getattr__(self, key):
        return self.__dict__.get(key, None)
 
my_scope = {'self': State()}

t = """
def init():
	if self.x == None:
		self.x = 0
def add_to_x():
	print(self.x)
	self.x += 1
init()
add_to_x()
"""

exec(t, my_scope)
exec(t, my_scope)
exec(t, my_scope)
exec(t, my_scope)
exec(t, my_scope)
exec(t, my_scope)