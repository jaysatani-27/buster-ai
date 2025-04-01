'use client';

import { BusterListColumn, BusterListRow } from '@/components/list';
import { BusterListVirtua } from '@/components/list/BusterList/BusterListVirtua';
import { faker } from '@faker-js/faker';
import { useEffect, useState } from 'react';

const generateRows = (numberToGenerate: number): BusterListRow[] => {
  return Array.from({ length: numberToGenerate }, (_, index) => ({
    id: faker.string.uuid(),
    data: {
      name: faker.person.fullName() + ` ${index}`,
      email: faker.internet.email(),
      phone: faker.phone.number()
    }
  }));
};
const rows: BusterListRow[] = Array.from({ length: 10 }, (_, index) => {
  const rows = generateRows(faker.number.int({ min: 10, max: 10 }));
  return [
    {
      id: `section-${index + 1}`,
      rowSection: {
        title: `Section ${index + 1}`,
        secondaryTitle: rows.length.toString()
      },
      data: {}
    },
    ...rows
  ];
}).flat();

const columns: BusterListColumn[] = [
  { title: 'Name', dataIndex: 'name' },
  { title: 'Email', dataIndex: 'email' },
  { title: 'Phone', dataIndex: 'phone' }
];

export default function ListTest() {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setTimeout(() => {
      setMounted(true);
    }, 100);
  }, []);

  if (!mounted) return null;

  return (
    <div className="h-[77vh] w-[66vw] border border-red-500 bg-white">
      <BusterListVirtua columns={columns} rows={rows} />
    </div>
  );
}
