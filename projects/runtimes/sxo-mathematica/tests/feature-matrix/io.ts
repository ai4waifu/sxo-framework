import { feature } from '@sxo/harness';

export const ioFeatures = [
    feature('Import', 'io')
        .unsupported('SILENT WRONG: Import["x.csv"] returns "x.csv" string')
        .effectful()
        .gap('import.strip', 'Import["x.csv"]', { expected: 'UnsupportedOperation', notes: 'must not strip to filename' })
        .done(),
    feature('Export', 'io')
        .unsupported('SILENT WRONG: Export["x.csv",1] returns 1')
        .effectful()
        .gap('export.strip', 'Export["x.csv", 1]', { expected: 'UnsupportedOperation' })
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
