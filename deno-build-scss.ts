// Build all sass files in project:
//
// deno -A deno-build-scss.ts --all
//
// Build a single file:
//
// deno -A deno-build-scss.ts style.scss style.css

import sass from "https://deno.land/x/denosass/mod.ts";
import postcss from "https://deno.land/x/postcss/mod.js";
import autoprefixer from "https://deno.land/x/postcss_autoprefixer/mod.js";
import postcssImport from "https://deno.land/x/postcss_import/mod.js";
import { parse } from "https://deno.land/std@0.149.0/path/mod.ts";

const file = Deno.args[0];
const fileParsed = parse(file);
const outfile = Deno.args[1];
if (!file) {
    console.log("Sass file is not given");
    Deno.exit(1);
}

// Build all sass files in project
if (file === "--all") {
    console.log("TODO");
    Deno.exit(1);
}

async function buildSassFile(file: string, outFile: string) {
    const inputFile = parse(file);
    console.log(`Building '${file}'...`);
    if (inputFile.dir) {
        Deno.chdir(inputFile.dir);
    }
    const sassed = sass([inputFile.base], {
        style: "expanded",
    }).to_string() as Map<string, string>;
    const css = sassed.get(inputFile.name) as string;
    if (!css) {
        console.log("css", css, sassed);
        throw new Error("Foo");
    }
    const posted = await postcss()
        .use(
            postcssImport({
                filter: (e: any) => {
                    console.log("e");
                    return false;
                },
            })
        )
        .use(autoprefixer())
        .process(css, {
            from: inputFile.base,
            to: inputFile.name + ".css",
        });
    if (outfile) {
        await Deno.writeTextFile(outFile, posted.css);
    }
}

if (fileParsed.name.startsWith("_")) {
    Deno.chdir("css");
    try {
        await buildSassFile("style.scss", "style.css");
        await buildSassFile("editor-style.scss", "editor-style.css");
    } catch (e) {
        console.error(e);
        Deno.exit(1);
    }

    Deno.exit(0);
}

try {
    await buildSassFile(file, outfile);
} catch (e) {
    console.error(e);
    Deno.exit(1);
}
