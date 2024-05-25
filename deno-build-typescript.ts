import * as esbuild from "https://deno.land/x/esbuild@v0.20.1/mod.js";
import { denoPlugins } from "https://deno.land/x/esbuild_deno_loader@0.9.0/mod.ts";

async function buildTsFile(file: string, outFile: string) {
    if (file.endsWith("interfaces.ts")) {
        console.error("Ignoring interfaces file: ", file);
        return;
    }
    if (file.endsWith(".test.ts")) {
        console.error("Ignoring test file: ", file);
        return;
    }
    if (!file) {
        console.error("Please provide a file to bundle.");
        return false;
    }

    if (!outFile) {
        console.error("Please provide an output file.");
        return false;
    }

    if (!file.endsWith(".ts") && !file.endsWith(".tsx")) {
        console.error("Please provide a .ts file to bundle.");
        console.error("Given file: ", file);
        return false;
    }

    if (!outFile.endsWith(".js")) {
        console.error("Please provide a .js output file.");
        console.error("Given file: ", outFile);
        return false;
    }

    await esbuild.build({
        // deno-lint-ignore no-explicit-any
        plugins: [...(denoPlugins() as any)],
        entryPoints: [file],
        outfile: outFile,
        bundle: true,
        format: "esm",
        // format: "iife",
        // jsx: "automatic",
        // jsxFactory: "wp.element.createElement",
        // treeShaking: true,

        // Minify is required for dead code elimination, tree shaking doesn't
        // remove: `if (false) { ... }` statements.
        // minifySyntax: true,
        // minifyWhitespace: true,
        // minify: true,
        define: {
            Deno: "false",
        },
    });

    // await esbuild.stop();
}

// Get first argument
const file = Deno.args[0];
const outFile = Deno.args[1];

try {
    await buildTsFile(file, outFile);
} catch (e) {
    console.error("Error building file: ", e);
}
