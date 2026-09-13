// Copy the current dirty source tree and add a capture-only data-source boundary.
// The working product source, user Desktop, and normal configuration are untouched.
import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import {fileURLToPath} from "node:url";

const project=path.resolve(path.dirname(fileURLToPath(import.meta.url)),"..");
const workspace=path.resolve(project,"../..");
const root=path.join(project,".capture","reviewed-build");
const version=process.argv[2]??"v1";
if(!/^v[0-9]+$/.test(version))throw Error("Expected a source snapshot version such as v2.");
const copy=path.join(root,`source-${version}`);
try {await fs.access(copy);throw new Error("Source snapshot already exists; choose a new version.");}
catch(error){if(error.code!=="ENOENT")throw error;}
await fs.mkdir(copy,{recursive:true});
const entries=["Cargo.toml","Cargo.lock","crates","vendor","tools","ui","locales"];
const records=[];
const hash=b=>crypto.createHash("sha256").update(b).digest("hex");
async function copyEntry(relative){
  const src=path.join(workspace,relative),dst=path.join(copy,relative);
  const stat=await fs.lstat(src);
  if(stat.isSymbolicLink())throw Error(`Unexpected source symlink: ${relative}`);
  if(stat.isDirectory()){
    await fs.mkdir(dst,{recursive:true});
    for(const name of (await fs.readdir(src)).sort())await copyEntry(path.join(relative,name));
  }else if(stat.isFile()){
    const bytes=await fs.readFile(src);
    await fs.writeFile(dst,bytes);
    records.push({path:relative.replaceAll("\\","/"),sha256:hash(bytes)});
  }
}
for(const entry of entries)await copyEntry(entry);
for(const record of records){
  if(hash(await fs.readFile(path.join(workspace,record.path)))!==record.sha256){
    throw Error(`Source changed during snapshot: ${record.path}. Use a fresh snapshot version.`);
  }
}
const shellFile=path.join(copy,"crates/platform/src/shell.rs");
const original=await fs.readFile(shellFile,"utf8");
const newline=original.includes("\r\n")?"\r\n":"\n";
const replacements=[
  {
    before:["pub fn user_desktop() -> Option<PathBuf> {","    known_folder(&FOLDERID_DESKTOP)","}"],
    after:[
      "pub fn user_desktop() -> Option<PathBuf> {",
      "    // Capture-only source snapshot: use an isolated demo directory.",
      "    #[cfg(debug_assertions)]",
      "    if let Some(root) = std::env::var_os(\"PECOFENCE_DEMO_DESKTOP\") {",
      "        let path = PathBuf::from(root);",
      "        assert!(path.is_absolute() && path.is_dir(), \"Invalid demo desktop root\");",
      `        let approved = PathBuf::from(r"${path.join(project,".capture","reviewed")}").canonicalize().expect("Missing approved fixture area");`,
      "        let resolved = path.canonicalize().expect(\"Cannot resolve demo desktop root\");",
      "        assert!(resolved.starts_with(&approved), \"Demo desktop escaped the approved fixture area\");",
      "        assert!(path.file_name() == Some(std::ffi::OsStr::new(\"desktop-fixture\")), \"Unexpected fixture directory name\");",
      "        return Some(path);",
      "    }",
      "    known_folder(&FOLDERID_DESKTOP)",
      "}",
    ],
  },
  {
    before:["pub fn public_desktop() -> Option<PathBuf> {","    known_folder(&FOLDERID_PUBLIC_DESKTOP)","}"],
    after:[
      "pub fn public_desktop() -> Option<PathBuf> {",
      "    #[cfg(debug_assertions)]",
      "    if std::env::var_os(\"PECOFENCE_DEMO_DESKTOP\").is_some() {",
      "        return None;",
      "    }",
      "    known_folder(&FOLDERID_PUBLIC_DESKTOP)",
      "}",
    ],
  },
];
let patched=original;
for(const item of replacements){
  const before=item.before.join(newline);
  if(patched.split(before).length!==2)throw Error("Expected resolver function changed; inspect source.");
  patched=patched.replace(before,item.after.join(newline));
}
await fs.writeFile(shellFile,patched);
const differences=[];
for(const record of records){
  const actual=hash(await fs.readFile(path.join(copy,record.path)));
  if(actual!==record.sha256)differences.push({path:record.path,before:record.sha256,after:actual});
}
if(differences.length!==1||differences[0].path!=="crates/platform/src/shell.rs")throw Error("Unexpected capture source differences");
const manifest={
  version,
  preparedAt:new Date().toISOString(),
  originalSourceFingerprint:hash(Buffer.from(JSON.stringify(records))),
  files:records, differences,
  description:"Only debug desktop-root resolution differs. Native grouping, tabs, Peek, rendering, and shell-open handlers are unchanged.",
};
await fs.writeFile(path.join(root,"source-manifest.json"),JSON.stringify(manifest,null,2)+"\n");
await fs.writeFile(path.join(root,`source-manifest-${version}.json`),JSON.stringify(manifest,null,2)+"\n");
await fs.writeFile(path.join(project,"review","capture-fixture-diff.txt"),
  replacements.map(r=>r.before.map(s=>"- "+s).concat(r.after.map(s=>"+ "+s)).join("\n")).join("\n\n")+"\n");
console.log(`Prepared ${records.length} source files at ${copy}`);
console.log(`Original source fingerprint: ${manifest.originalSourceFingerprint}`);
console.log("Exactly one source file differs, with two guarded desktop-root overrides.");
