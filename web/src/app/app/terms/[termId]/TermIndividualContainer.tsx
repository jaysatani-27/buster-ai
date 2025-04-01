'use client';

import React from 'react';
import { AppContentHeader } from '../../_components/AppContentHeader';
import { TermIndividualHeader } from './_TermIndividualHeader';
import { TermIndividualContent } from './_TermIndividualContent';
import { AppSplitter } from '@/components/layout';
import { AppContent } from '../../_components/AppContent';
import { useMount } from 'ahooks';
import { TermIndividualHeaderSider } from './_TermIndividualHeaderSider';
import { TermIndividualContentSider } from './_TermIndividualContentSider';
import { useTermsContextSelector } from '@/context/Terms';

export const TermIndividualContainer: React.FC<{
  termPageIdLayout: string[];
  termId: string;
}> = ({ termPageIdLayout, termId }) => {
  const loadedTermsList = useTermsContextSelector((x) => x.loadedTermsList);
  const getInitialTerms = useTermsContextSelector((x) => x.getInitialTerms);

  useMount(() => {
    getInitialTerms();
  });

  return (
    <AppSplitter
      defaultLayout={termPageIdLayout}
      rightPanelMinSize={'280px'}
      rightPanelMaxSize={'400px'}
      autoSaveId="term-page"
      preserveSide="right"
      leftChildren={
        <>
          <TermIndividualHeader termId={termId} />
          <TermIndividualContent termId={termId} />
        </>
      }
      rightHidden={!loadedTermsList}
      rightChildren={
        <>
          <AppContentHeader>
            <TermIndividualHeaderSider />
          </AppContentHeader>
          <AppContent>
            <TermIndividualContentSider termId={termId} />
          </AppContent>
        </>
      }
    />
  );
};
