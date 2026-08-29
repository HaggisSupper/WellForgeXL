const clone = (value) => structuredClone(value);

export function mockAdapter(initial = {}, options = {}) {
  const cells = new Map(Object.entries(initial).map(([key, value]) => [key, clone(value)]));
  const formulaCells = new Set(options.formulas ?? []);
  let writeCount = 0;

  return {
    writes: [],
    read(sheet, address) {
      return clone(cells.get(`${sheet}!${address}`));
    },
    capture(sheet, address) {
      if (options.failOnCapture === `${sheet}!${address}`) throw new Error(`simulated capture failure at ${sheet}!${address}`);
      return { sheet, address, value: clone(cells.get(`${sheet}!${address}`)) };
    },
    write(sheet, address, value) {
      writeCount += 1;
      if (options.failOnWrite === writeCount) throw new Error(`simulated write failure at ${sheet}!${address}`);
      const key = `${sheet}!${address}`;
      cells.set(key, clone(value));
      this.writes.push({ sheet, address, value: clone(value) });
    },
    restore(changeSet) {
      for (const change of changeSet) cells.set(`${change.sheet}!${change.address}`, clone(change.value));
    },
    isFormula(sheet, address) {
      return formulaCells.has(`${sheet}!${address}`);
    },
  };
}
