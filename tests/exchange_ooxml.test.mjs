import test from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs/promises';
import JSZip from 'jszip';

import { buildApi7gWorkbook } from '../src/build_api7g.mjs';
import { buildBhaWorkbook } from '../src/build_bha.mjs';
import { buildDirectionalWorkbook } from '../src/build_directional.mjs';
import { buildHydraulicsWorkbook } from '../src/build_hydraulics.mjs';
import { buildTorqueDragWorkbook } from '../src/build_torque_drag.mjs';
import { exportExchangeXlsx } from '../src/exchange/export_exchange_xlsx.mjs';

const BUILDERS = {
  api7g: buildApi7gWorkbook,
  hydraulics: buildHydraulicsWorkbook,
  torqueDrag: buildTorqueDragWorkbook,
  bha: buildBhaWorkbook,
  directional: buildDirectionalWorkbook,
};

function sheetTag(xml, name) {
  return xml.match(new RegExp(`<x:sheet\\b[^>]*name="${name}"[^>]*/>`))?.[0];
}

function relationshipTarget(workbookXml, relationshipsXml, name) {
  const relationId = sheetTag(workbookXml, name)?.match(/r:id="([^"]+)"/)?.[1];
  assert.ok(relationId, name);
  const relationship = [...relationshipsXml.matchAll(/<Relationship\b[^>]*\/>/g)]
    .map(([value]) => value)
    .find((value) => value.match(/\bId="([^"]+)"/)?.[1] === relationId);
  const target = relationship?.match(/\bTarget="([^"]+)"/)?.[1];
  assert.ok(target, name);
  return target.startsWith('/xl/') ? target.slice(1) : `xl/${target.replace(/^\//, '')}`;
}

test('every serialized XLSX hides Exchange State and protects map and state sheets', async () => {
  for (const [kind, build] of Object.entries(BUILDERS)) {
    const zip = await JSZip.loadAsync((await exportExchangeXlsx(build())).data);
    const workbookXml = await zip.file('xl/workbook.xml').async('string');
    assert.match(workbookXml, /<x:sheet[^>]+name="Exchange State"[^>]+state="hidden"/, kind);
    assert.match(workbookXml, /<x:sheet[^>]+name="Calc"[^>]+state="hidden"/, kind);
    assert.doesNotMatch(sheetTag(workbookXml, 'Exchange Map') ?? '', /state="hidden"/, kind);
    assert.doesNotMatch(sheetTag(workbookXml, 'Exchange Buffer') ?? '', /state="hidden"/, kind);
    const relationships = await zip.file('xl/_rels/workbook.xml.rels').async('string');
    for (const name of ['Exchange Map', 'Exchange State']) {
      const xml = await zip.file(relationshipTarget(workbookXml, relationships, name)).async('string');
      assert.match(xml, /<x:sheetProtection\b[^>]*sheet="1"/, `${kind}: ${name}`);
      const sheetDataEnd = xml.indexOf('</x:sheetData>');
      const protection = xml.indexOf('<x:sheetProtection');
      const mergeCells = xml.indexOf('<x:mergeCells');
      assert.ok(sheetDataEnd >= 0 && protection > sheetDataEnd, `${kind}: ${name} protection must follow sheetData`);
      if (mergeCells >= 0) assert.ok(protection < mergeCells, `${kind}: ${name} protection must precede mergeCells`);
      if (name === 'Exchange Map') {
        const protectedRanges = xml.indexOf('<x:protectedRanges');
        assert.ok(protectedRanges > protection, `${kind}: editable range must follow protection`);
        if (mergeCells >= 0) assert.ok(protectedRanges < mergeCells, `${kind}: editable range must precede mergeCells`);
        assert.match(xml, /<x:protectedRange\b[^>]*name="ExchangeMapDocumentation"[^>]*sqref="A3:M3"/);
      } else {
        assert.doesNotMatch(xml, /<x:protectedRanges\b/, `${kind}: Exchange State must remain fully protected`);
      }
    }
  }
});

test('exchange export is in-process, Windows-portable, and byte deterministic', async () => {
  const source = await fs.readFile(new URL('../src/exchange/export_exchange_xlsx.mjs', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /node:child_process|execFile\s*\(|\b(?:unzip|zip)\s*,?\s*\[/);
  assert.match(source, /from 'jszip'/);

  const workbook = buildApi7gWorkbook();
  const first = await exportExchangeXlsx(workbook);
  const second = await exportExchangeXlsx(workbook);
  const digest = ({ data }) => crypto.createHash('sha256').update(data).digest('hex');
  assert.equal(digest(first), digest(second));
});
