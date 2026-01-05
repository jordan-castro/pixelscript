#include "pixel_script.h"

// ========================== AI Generated Code (START) ==========================

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Define the Struct
typedef struct Person {
    char *name;
    int age;

    // Function pointer to act as a "method"
    void (*print_info)(struct Person *self);
} Person;

// Implementation of the print method
void print_person_info(Person *self) {
    if (self != NULL) {
        printf("My name is: %s, and I am %d years old\n", self->name, self->age);
    }
}

// "Getter" for Name
const char* get_name(Person *self) {
    return self->name;
}

// "Getter" for Age
int get_age(Person *self) {
    return self->age;
}

// "Setter" for Name
void set_name(Person *self, const char* name) {
    // Free old name
    free(self->name);
    self->name = strdup(name);
} 

// Constructor-like function to initialize the struct
Person* create_person(const char *name, int age) {
    Person *p = malloc(sizeof(Person));
    p->name = strdup(name); // Duplicate string to ensure it persists
    p->age = age;
    return p;
}

// Destructor to free memory
void destroy_person(Person *p) {
    free(p->name);
    free(p);
}

// ========================== AI Generated Code (END) ==========================

Var* ps_set_name(uintptr_t argc, struct Var **argv, void *opaque) {
    Var* object = argv[1];
    // Name is either argv[2] or argv[3] due to the "self" variable that gets passed in via Lua.
    Var* name = argv[3]; // In this example we use LUA, so we will stick to it.

    Person* p = pixelscript_var_get_host_object(object);
    const char* new_name = pixelscript_var_get_string(name);

    set_name(p, new_name);

    pixelscript_free_str(new_name);

    return pixelscript_var_newnull();
}

Var* ps_get_name(uintptr_t argc, struct Var **argv, void *opaque) {
    Var* object = argv[1];
    
    Person* p = pixelscript_var_get_host_object(object);

    return pixelscript_var_newstring(p->name);
}

Var* ps_get_age(uintptr_t argc, struct Var **argv, void *opaque) { 
    Var* object = argv[1];

    Person* p = pixelscript_var_get_host_object(object);

    return pixelscript_var_newi64(p->age);
}

Var* ps_greet(uintptr_t argc, struct Var **argv, void *opaque) { 
    Var *object = argv[1];

    Person* p = pixelscript_var_get_host_object(object);

    print_person_info(p);

    return pixelscript_var_newnull();
}

Var* new_person(uintptr_t argc, struct Var **argv, void *opaque) {
    // Assume 1 and 2 are name and age
    Var* name = argv[1];
    Var* age_var = argv[2];

    // Get name string
    const char* name_str = pixelscript_var_get_string(name);
    int age = pixelscript_var_get_i64(age_var);

    // Create person
    Person* p = create_person(name_str, age);

    // Free name
    pixelscript_free_str(name_str);

    // Create new object
    PixelObject* object = pixelscript_new_object(p, destroy_person);
    // Add methods
    pixelscript_object_add_callback(object, "set_name", ps_set_name, NULL);
    pixelscript_object_add_callback(object, "get_name", ps_get_name, NULL);
    pixelscript_object_add_callback(object, "get_age", ps_get_age, NULL);
    pixelscript_object_add_callback(object, "greet", ps_greet, NULL);

    // Return object
    return pixelscript_var_newhost_object(object);
}

int main() {
    pixelscript_initialize();

    // Set the new_person object
    pixelscript_add_object("Person", new_person, NULL);
    const char* script = "local p = Person('Jordan', 23)\n"
                         "p:greet()\n"
                         "p:set_name('Jordan Castro')\n"
                         "p:greet()\n"
                         "p:set_name('Jordan Castro + ' .. p:get_age())\n"
                         "p:greet()\n";
    pixelscript_exec_lua(script, "<ctest>");

    pixelscript_finalize();

    return 0;
}