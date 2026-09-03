import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import path from 'node:path';

const root = path.resolve('projects/runtimes/sxo-mathematica');
const require = createRequire(path.join(root, 'package.json'));

async function main() {
  let Mathematica;
  try {
    ({ Mathematica } = require('./dist/index.js'));
  } catch {
    const mod = await import(pathToFileURL(path.join(root, 'dist/index.js')).href);
    Mathematica = mod.Mathematica;
  }
  const m = new Mathematica();
  const cases = [
    'If[1==1,7,8]',
    'Hold[1+1]',
    'HoldForm[1+1]',
    '{1,2,3}[[0]]',
    '{1,2,3}[[2]]',
    'Part[{1,2,3},0]',
    'Import["x.csv"]',
  ];
  for (const s of cases) {
    try {
      console.log(JSON.stringify(s), '=>', m.evaluate(s).toWolfram());
    } catch (e) {
      console.log(JSON.stringify(s), 'ERR', String(e));
    }
  }
}

main();
