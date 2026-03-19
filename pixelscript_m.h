#ifndef PIXEL_SCRIPT_M_H
#define PIXEL_SCRIPT_M_H

// Helpful to not have to write out the method everytime.
#define PXS_HANDLER(name) pxs_Var* name(pxs_VarT args)

// Helpful to not have to write out pxs_listget(args, index).
#define PXS_ARG(index) pxs_listget(args, index + 1)

// Helpful to not have to write out pxs_listlen(args). Does not include Runtime.
#define PXS_ARGC() pxs_listlen(args) - 1

// Get the current runtime via macro
#define PXS_RT() pxs_listget(args, 0)

// Helpful to not have to write out pxs_getint(PXS_ARG(0)).
#define PXS_GET_RT() pxs_getint(pxs_listget(args, 0))

// Helpful to not have to write out pxs_newint(runtime).
#define PXS_NEW_RT(runtime) pxs_newint(runtime)

// Get self
#define PXS_SELF() pxs_listget(args, 1)

#endif // PIXEL_SCRIPT_M_H