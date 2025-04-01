import React from 'react';
import { ExtraProps } from 'react-markdown';
import { AppCodeBlock } from './AppCodeBlock/AppCodeBlock';
import { TextPulseLoader } from '@/components';
import { Element } from 'hast';
import { AppMarkdownComponentType, CustomComponents } from './config';
import { BusterTimestamp } from './AppMarkdown_BusterTimestamp';
import { BusterSuggestion } from './AppMarkdown_BusterSuggestion';
import { BusterDatasets } from './AppMarkdown_BusterDatasets';

export const commonStreamingCheck = (
  endLine?: number,
  startLine?: number,
  lastTrackedLine?: number
): boolean => {
  const isLineNumber = typeof endLine === 'number' && typeof lastTrackedLine === 'number';
  return isLineNumber && endLine === lastTrackedLine && startLine === lastTrackedLine;
};

export interface ExtraPropsExtra extends ExtraProps {
  numberOfLineMarkdown: number;
}

export const CommonPulseLoader: React.FC<{
  showLoader: boolean;
  numberOfLineMarkdown: number;
  node?: Element;
}> = ({ showLoader, numberOfLineMarkdown, node }) => {
  const showStreamingLoader =
    node &&
    showLoader &&
    commonStreamingCheck(
      node?.position?.end.line,
      node?.position?.start.line,
      numberOfLineMarkdown
    );

  if (showStreamingLoader) {
    return <TextPulseLoader />;
  }
  return null;
};

export const CustomCode: React.FC<
  {
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
    className?: string;
  } & ExtraPropsExtra
> = ({ children, markdown, showLoader, className, node, ...rest }) => {
  const matchRegex = /language-(\w+)/.exec(className || '');
  const language = matchRegex ? matchRegex[1] : undefined;
  const showStreamingLoader = showLoader && node?.position?.end.line === rest.numberOfLineMarkdown;

  return (
    <AppCodeBlock language={language} showLoader={showStreamingLoader}>
      {children}
    </AppCodeBlock>
  );
};

export const CustomParagraph: React.FC<
  {
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
  } & ExtraPropsExtra
> = ({ children, markdown, ...rest }) => {
  if (Array.isArray(children)) {
    return (
      <p className="nate-rulez">
        {children}
        <CommonPulseLoader {...rest} />
      </p>
    );
  }

  //weird bug where all web components are rendered as p
  //web components are objects
  if (typeof children === 'object') {
    return <>{children}</>;
  }

  return (
    <p className="">
      {children}
      <CommonPulseLoader {...rest} />
    </p>
  );
};

export const CustomHeading: React.FC<
  {
    level: 1 | 2 | 3 | 4 | 5 | 6;
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
    numberOfLineMarkdown: number;
  } & ExtraPropsExtra
> = ({ level, children, markdown, ...rest }) => {
  const HeadingTag = `h${level}` as any;

  return (
    <HeadingTag>
      {children}
      <CommonPulseLoader {...rest} />
    </HeadingTag>
  );
};

export const CustomList: React.FC<
  {
    ordered?: boolean;
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
  } & ExtraPropsExtra
> = ({ ordered, children, markdown, ...rest }) => {
  const ListTag = ordered ? 'ol' : 'ul';

  return (
    <ListTag>
      {children}
      <CommonPulseLoader {...rest} />
    </ListTag>
  );
};

export const CustomListItem: React.FC<
  {
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
  } & ExtraPropsExtra
> = ({ children, markdown, ...rest }) => {
  return (
    <li>
      {children}
      <CommonPulseLoader {...rest} />
    </li>
  );
};

export const CustomBlockquote: React.FC<
  {
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
  } & ExtraPropsExtra
> = ({ children, markdown, ...rest }) => {
  return (
    <blockquote>
      {children}
      <CommonPulseLoader {...rest} />
    </blockquote>
  );
};

export const CustomTable: React.FC<
  {
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
  } & ExtraPropsExtra
> = ({ children, markdown, ...rest }) => {
  return (
    <table>
      {children}
      <CommonPulseLoader {...rest} />
    </table>
  );
};

export const CustomSpan: React.FC<
  {
    children?: React.ReactNode;
    markdown: string;
    showLoader: boolean;
  } & ExtraPropsExtra
> = ({ children, markdown, ...rest }) => {
  return <span>{children}</span>;
};

export const AppMarkdownCustomComponents = (customProps: ExtraPropsExtra): CustomComponents => {
  return {
    [AppMarkdownComponentType.BusterTimestamp]: (props) => (
      <BusterTimestamp {...props} {...customProps} />
    ),
    [AppMarkdownComponentType.BusterSuggestion]: (props) => (
      <BusterSuggestion {...props} {...customProps} />
    ),
    [AppMarkdownComponentType.BusterDatasets]: (props) => (
      <BusterDatasets {...props} {...customProps} />
    )
  };
};
