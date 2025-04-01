'use client';

import { useRef } from 'react';
import { Virtualizer } from 'virtua';
import React from 'react';
import { useMount } from 'ahooks';

const headerHeight = 300;

const ItemSwag = React.memo(({ index }: { index: number }) => {
  return <div className="h-[48px] border bg-red-200">Swag {index}</div>;
});
ItemSwag.displayName = 'ItemSwag';

const createRows = (numberToGenerate: number) => {
  return Array.from({ length: numberToGenerate }, (_, index) => {
    return <ItemSwag key={index} index={index} />;
  });
};

export default function ListTest2() {
  const ref = useRef<HTMLDivElement>(null);
  const outerPadding = 30;
  const innerPadding = 50;

  return (
    <div
      ref={ref}
      style={{
        width: '100%',
        height: '100vh',
        overflowY: 'auto',
        // opt out browser's scroll anchoring on header/footer because it will conflict to scroll anchoring of virtualizer
        overflowAnchor: 'none'
      }}>
      <div
        style={{
          backgroundColor: 'burlywood',
          padding: outerPadding
        }}>
        <div
          style={{
            backgroundColor: 'steelblue',
            padding: innerPadding
          }}>
          <Virtualizer scrollRef={ref} startMargin={outerPadding + innerPadding}>
            {createRows(1000)}
          </Virtualizer>
        </div>
      </div>
    </div>
  );
}
