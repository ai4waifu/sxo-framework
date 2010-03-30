import { feature } from '@sxo/harness';

export const ioFeatures = [
    feature('Import', 'io')
        .unsupported('unevaluated Import["x.csv"] (no I/O / UnsupportedOperation yet)')
        .effectful()
        .gap('import.strip', 'Import["x.csv"]', {
            expected: 'UnsupportedOperation',
            notes: 'stays Import["x.csv"]; no longer silently returns the filename string',
        })
        .done(),
    feature('Export', 'io')
        .unsupported('unevaluated Export["x.csv", 1] (no I/O / UnsupportedOperation yet)')
        .effectful()
        .gap('export.strip', 'Export["x.csv", 1]', {
            expected: 'UnsupportedOperation',
            notes: 'stays Export["x.csv", 1]; no longer silently returns 1',
        })
        .done(),
    feature('FileNameJoin', 'io').unsupported().pure().gap('filenamejoin.ab', 'FileNameJoin[{"a", "b"}]', { expected: '...' }).done(),
    feature('ExportString', 'io')
        .unsupported()
        .pure()
        .gap('exportstring.csv', 'ExportString[{{1, 2}}, "CSV"]', { expected: '"1,2\\n"' })
        .done(),
    feature('ImportString', 'io').unsupported().pure().gap('importstring.csv', 'ImportString["1,2", "CSV"]', { expected: '{{1, 2}}' }).done(),
    feature('FileIO', 'io').planned().effectful().gap('file.absolute', 'AbsoluteFileName["."]', { expected: '...' }).done(),
    feature('Network', 'io').planned().effectful().gap('network.hostlookup', 'HostLookup["localhost"]', { expected: '...' }).done(),
];
