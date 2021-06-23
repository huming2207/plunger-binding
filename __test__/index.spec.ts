import test from 'ava';

import { listAllProbes } from '../index';

test('List all probes', (t) => {
    const result = listAllProbes();
    console.log(result);
    t.assert(true);
})

