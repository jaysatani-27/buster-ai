import { RowInput } from 'jspdf-autotable';
import { timeout } from './timeout';

export async function exportJSONToCSV(
  data: Record<string, string | null | Date | number>[],
  fileName: string = 'data'
) {
  if (data.length === 0) {
    throw new Error('No data to export');
  }

  // Get all unique headers from all rows
  const headers = Array.from(new Set(data.flatMap(Object.keys)));

  // Create CSV content
  let csvContent = headers.join(',') + '\n';

  data.forEach((row) => {
    const rowValues = headers.map((header) => {
      const value = row[header];
      if (value === null || value === undefined) {
        return '';
      }
      if (typeof value === 'string') {
        // Escape quotes and wrap in quotes if contains comma
        const escapedValue = value.replace(/"/g, '""');
        return escapedValue.includes(',') ? `"${escapedValue}"` : escapedValue;
      }
      if (value instanceof Date) {
        return value.toISOString();
      }
      return String(value);
    });
    csvContent += rowValues.join(',') + '\n';
  });

  // Create Blob and download
  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  downloadFile(`${fileName}.csv`, blob);
}

export async function exportToCsv<R, SR>(gridElement: HTMLDivElement, fileName: string) {
  const { head, body, foot } = await getGridContent(gridElement);
  const content = [...head, ...body, ...foot]
    .map((cells) => cells.map(serialiseCellValue).join(','))
    .join('\n');

  downloadFile(fileName, new Blob([content], { type: 'text/csv;charset=utf-8;' }));
}

async function getGridContent<R, SR>(gridElement: HTMLDivElement) {
  const grid = gridElement as HTMLDivElement;
  const hasrdgClass = grid.classList.contains('rdg');
  if (!hasrdgClass) {
    throw new Error('Element is not a valid ReactDataGrid instance');
  }
  return {
    head: getRows('.rdg-header-row'),
    body: getRows('.rdg-row:not(.rdg-summary-row)'),
    foot: getRows('.rdg-summary-row')
  };

  function getRows(selector: string) {
    return Array.from(grid.querySelectorAll<HTMLDivElement>(selector)).map((gridRow) => {
      return Array.from(gridRow.querySelectorAll<HTMLDivElement>('.rdg-cell')).map(
        (gridCell) => gridCell.innerText
      );
    });
  }
}

function serialiseCellValue(value: unknown) {
  if (typeof value === 'string') {
    const formattedValue = value.replace(/"/g, '""');
    return formattedValue.includes(',') ? `"${formattedValue}"` : formattedValue;
  }
  return value;
}

function downloadFile(fileName: string, data: Blob) {
  const downloadLink = document.createElement('a');
  downloadLink.download = fileName;
  const url = URL.createObjectURL(data);
  downloadLink.href = url;
  downloadLink.click();
  setTimeout(() => {
    URL.revokeObjectURL(url);
    downloadLink.remove();
  }, 100);
}

export async function exportGridToPdf<R, SR>(gridElement: HTMLDivElement, fileName: string) {
  const [{ jsPDF }, autoTable, { head, body, foot }] = await Promise.all([
    import('jspdf'),
    (await import('jspdf-autotable')).default,
    await getGridContent(gridElement)
  ]);
  const doc = new jsPDF({
    orientation: 'l',
    unit: 'px'
  });

  autoTable(doc, {
    head,
    body,
    foot,
    horizontalPageBreak: true,
    styles: { cellPadding: 1.5, fontSize: 8, cellWidth: 'wrap' },
    tableWidth: 'wrap'
  });

  doc.save(fileName);
}

export async function exportJSONToPDF(
  data: Record<string, string | null | Date | number>[],
  fileName: string = 'data'
) {
  const [{ jsPDF }, autoTable] = await Promise.all([
    import('jspdf'),
    (await import('jspdf-autotable')).default
  ]);
  const doc = new jsPDF();

  const headSet = data.reduce<Set<string>>((acc, row) => {
    Object.keys(row).forEach((key) => acc.add(key));
    return acc;
  }, new Set());
  const head: RowInput[] = [Array.from(headSet)];
  const body: RowInput[] = data.map((row) =>
    Object.values(row).map((value) => String(value || ''))
  );

  autoTable(doc, {
    head,
    body,
    horizontalPageBreak: true,
    styles: { cellPadding: 1.5, fontSize: 8, cellWidth: 'wrap' },
    tableWidth: 'wrap'
  });

  doc.save(fileName);
}

export async function exportElementToImage(element: HTMLElement) {
  //@ts-ignore
  const domToImage = (await import('dom-to-image').then((m) => m.default)) as any;
  const dataUrl = (await domToImage.toPng(element)) as string;
  return dataUrl;
}

export async function downloadImageData(imageData: string, fileName: string) {
  const link = document.createElement('a');
  link.href = imageData;
  link.download = fileName;
  link.click();
  await timeout(1);
  link.remove();
  URL.revokeObjectURL(imageData);
}
