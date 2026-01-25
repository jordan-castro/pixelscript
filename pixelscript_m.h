#ifndef PIXEL_SCRIPT_M_H
#define PIXEL_SCRIPT_M_H

// Helpful to not have to write out the method everytime.
#define PXS_HANDLER(name) pxs_Var* name(pxs_VarT args, pxs_Opaque opaque)

// Helpful to not have to write out pxs_listget(args, index).
#define PXS_ARG(index) pxs_listget(args, index)

// Helpful to not have to write out pxs_listlen(args).
#define PXS_ARGC() pxs_listlen(args)

// Helpful to not have to write out pxs_getint(PXS_ARG(0)).
#define PXS_GET_RT() pxs_getint(PXS_ARG(0))

// Helpful to not have to write out pxs_newint(runtime).
#define PXS_NEW_RT(runtime) pxs_newint(runtime)

#endif // PIXEL_SCRIPT_M_H