// gentype's `.gen.ts` files import their compiled ReScript runtime (`.res.mjs`),
// which ships no type declarations. The generated code already casts every value
// through `as any`, so treating the runtime modules as untyped is correct.
declare module "*.res.mjs";
