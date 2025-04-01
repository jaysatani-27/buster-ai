import { formatLabel } from './columnFormatter';

describe('formatLabel', () => {
  describe('number formatting', () => {
    it('should format numbers with default settings', () => {
      expect(formatLabel(1234.567, { columnType: 'number', style: 'number' })).toBe('1,234.57');
    });

    it('should format numbers with custom fraction digits', () => {
      expect(
        formatLabel(1234.567, {
          columnType: 'number',
          style: 'number',
          minimumFractionDigits: 3,
          maximumFractionDigits: 3
        })
      ).toBe('1,234.567');
    });

    it('should format currency values', () => {
      expect(
        formatLabel(1234.56, {
          columnType: 'number',
          style: 'currency',
          currency: 'USD'
        })
      ).toBe('$1,234.56');

      expect(
        formatLabel(1234.56, {
          columnType: 'number',
          style: 'currency',
          currency: 'EUR'
        })
      ).toBe('€1,234.56');
    });

    it('should handle prefix and suffix', () => {
      expect(
        formatLabel(1234, {
          columnType: 'number',
          style: 'number',
          prefix: 'Pre-',
          suffix: '-Post'
        })
      ).toBe('Pre-1,234-Post');
    });

    it('should handle missing values', () => {
      expect(
        formatLabel(null, {
          columnType: 'number',
          style: 'number',
          replaceMissingDataWith: 0
        })
      ).toBe('0');

      expect(
        formatLabel(undefined, {
          columnType: 'number',
          style: 'number',
          replaceMissingDataWith: null
        })
      ).toBe('null');
    });

    it('should format percentages', () => {
      expect(
        formatLabel(0.1234, {
          columnType: 'number',
          style: 'percent'
        })
      ).toBe('0.12%');
    });

    it('should apply multiplier', () => {
      expect(
        formatLabel(100, {
          columnType: 'number',
          style: 'number',
          multiplier: 2
        })
      ).toBe('200');
    });

    it('should handle replaceMissingDataWith with custom string value', () => {
      expect(
        formatLabel(null, {
          columnType: 'number',
          style: 'number',
          replaceMissingDataWith: 'N/A'
        })
      ).toBe('N/A');
    });

    it('should handle replaceMissingDataWith with custom string value', () => {
      expect(
        formatLabel(null, {
          columnType: 'number',
          style: 'number',
          replaceMissingDataWith: ''
        })
      ).toBe('');
    });

    it('should handle replaceMissingDataWith with custom number value', () => {
      expect(
        formatLabel(undefined, {
          columnType: 'number',
          style: 'number',
          replaceMissingDataWith: '-999'
        })
      ).toBe('-999');
    });

    it('should apply formatting when using replaceMissingDataWith', () => {
      expect(
        formatLabel(null, {
          columnType: 'number',
          style: 'currency',
          currency: 'USD',
          replaceMissingDataWith: null
        })
      ).toBe('null');
    });

    // it('should handle replaceMissingDataWith with empty string', () => {
    //   expect(
    //     formatLabel(undefined, {
    //       columnType: 'number',
    //       style: 'number',
    //       replaceMissingDataWith: ''
    //     })
    //   ).toBe('');
    // });
  });

  describe('string formatting', () => {
    it('should format strings', () => {
      expect(formatLabel('test_string', { columnType: 'string', style: 'string' })).toBe(
        'test_string'
      );
    });
    it('should handle null/undefined strings', () => {
      expect(formatLabel(null, { columnType: 'string', style: 'string' })).toBe('null');
      expect(formatLabel(undefined, { columnType: 'string', style: 'string' })).toBe('null');
    });
    it('should make labels human readable when specified', () => {
      expect(
        formatLabel(
          'test_string',
          {
            columnType: 'string',
            style: 'string',
            makeLabelHumanReadable: true
          },
          true
        )
      ).toBe('Test String');
    });

    it('should handle undefined strings', () => {
      expect(formatLabel(undefined, { columnType: 'string', style: 'string' })).toBe('null');
    });

    it('should handle null strings', () => {
      expect(formatLabel(null, { columnType: 'string', style: 'string' })).toBe('null');
    });

    it('should handle empty strings', () => {
      expect(formatLabel('', { columnType: 'string', style: 'string' })).toBe('');
    });

    it('should handle null strings', () => {
      expect(formatLabel(null, { columnType: 'string', style: 'number' })).toBe('null');
    });

    it('should handle empty strings', () => {
      expect(formatLabel('', { columnType: 'string', style: 'number' })).toBe('');
    });

    it('should handle replaceMissingDataWith for null values with number', () => {
      expect(
        formatLabel(null, {
          columnType: 'string',
          style: 'string',
          replaceMissingDataWith: 0
        })
      ).toBe('0');
    });

    it('should handle replaceMissingDataWith for undefined values with string', () => {
      expect(
        formatLabel(undefined, {
          columnType: 'string',
          style: 'string',
          replaceMissingDataWith: 'N/A'
        })
      ).toBe('N/A');
    });

    it('should handle replaceMissingDataWith for null values with empty string', () => {
      expect(
        formatLabel(null, {
          columnType: 'string',
          style: 'string',
          replaceMissingDataWith: ''
        })
      ).toBe('');
    });

    it('should handle replaceMissingDataWith for null values with null', () => {
      expect(
        formatLabel(null, {
          columnType: 'string',
          style: 'string',
          replaceMissingDataWith: null
        })
      ).toBe('null');
    });
  });

  describe('date formatting', () => {
    const testDate = new Date('2024-03-14T12:00:00Z');
    it('should format dates with default format', () => {
      expect(
        formatLabel(testDate, {
          columnType: 'date',
          style: 'date'
        })
      ).toMatch(/Mar(ch)? 14, 2024/);
    });
    it('should format dates with custom format', () => {
      expect(
        formatLabel(testDate, {
          columnType: 'date',
          style: 'date',
          dateFormat: 'YYYY-MM-DD'
        })
      ).toBe('2024-03-14');
    });
    it('should handle UTC dates', () => {
      expect(
        formatLabel(testDate, {
          columnType: 'date',
          style: 'date',
          dateFormat: 'YYYY-MM-DD HH:mm',
          isUTC: true
        })
      ).toBe('2024-03-14 12:00');
    });
    it('should convert numbers to date units when specified', () => {
      const currentYear = new Date().getFullYear();
      expect(
        formatLabel(1, {
          columnType: 'date',
          style: 'date',
          convertNumberTo: 'month_of_year'
        })
      ).toMatch(/January/);
      expect(
        formatLabel(1, {
          columnType: 'date',
          style: 'date',
          convertNumberTo: 'day_of_week'
        })
      ).toMatch(/Monday/);
      expect(
        formatLabel(1, {
          columnType: 'date',
          style: 'date',
          convertNumberTo: 'quarter'
        })
      ).toMatch(`${currentYear} QQ`);
    });

    it('should handle null/undefined dates', () => {
      expect(formatLabel(null, { columnType: 'date', style: 'date' })).toBe('null');
      expect(formatLabel(undefined, { columnType: 'date', style: 'date' })).toBe('null');
    });

    it('should handle empty dates', () => {
      expect(formatLabel('', { columnType: 'date', style: 'date' })).toBe('');
    });

    it('should handle invalid dates', () => {
      expect(formatLabel('invalid date', { columnType: 'date', style: 'date' })).toBe(
        'invalid date'
      );
    });

    it('should handle replaceMissingDataWith', () => {
      expect(
        formatLabel(null, { columnType: 'date', style: 'date', replaceMissingDataWith: 'N/A' })
      ).toBe('N/A');
    });
  });
});
