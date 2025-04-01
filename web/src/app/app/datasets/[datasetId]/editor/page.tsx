import { EditorContent } from './EditorContent';
import React from 'react';

const defaultLayout: [string, string] = ['auto', '170px'];

export default async function Page({ params }: { params: { datasetId: string } }) {
  return <EditorContent defaultLayout={defaultLayout} />;
}
