import { createStyles } from 'antd-style';
import React, { memo, useMemo } from 'react';
import ReactMarkdown, { Components } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw'; // For rendering HTML
import {
  CustomCode,
  CustomHeading,
  CustomListItem,
  CustomParagraph,
  CustomBlockquote,
  CustomSpan,
  AppMarkdownCustomComponents
} from './AppMarkdownCommon';
import { useMemoizedFn } from 'ahooks';

const _AppMarkdown: React.FC<{
  markdown: string | null;
  showLoader?: boolean;
  className?: string;
}> = ({ markdown = '', showLoader = false, className = '' }) => {
  const { cx, styles } = useStyles();

  const currentMarkdown = markdown || '';
  const commonProps = useMemo(() => {
    const numberOfLineMarkdown = currentMarkdown.split('\n').length;
    return {
      markdown: currentMarkdown,
      showLoader,
      numberOfLineMarkdown
    };
  }, [currentMarkdown, showLoader]);

  const text = useMemoizedFn((props: React.SVGTextElementAttributes<SVGTextElement>) => (
    <CustomParagraph {...props} {...commonProps} />
  ));
  const code = useMemoizedFn((props) => <CustomCode {...props} {...commonProps} />);
  const heading = useMemoizedFn((props) => <CustomHeading {...props} {...commonProps} />);
  const listItem = useMemoizedFn((props) => <CustomListItem {...props} {...commonProps} />);
  const blockquote = useMemoizedFn((props) => <CustomBlockquote {...props} {...commonProps} />);
  const span = useMemoizedFn((props) => <CustomSpan {...props} {...commonProps} />);
  const li = useMemoizedFn((props) => <CustomListItem {...props} {...commonProps} />);
  const p = useMemoizedFn((props) => <CustomParagraph {...props} {...commonProps} />);
  const h1 = useMemoizedFn((props) => <CustomHeading level={1} {...props} {...commonProps} />);
  const h2 = useMemoizedFn((props) => <CustomHeading level={2} {...props} {...commonProps} />);
  const h3 = useMemoizedFn((props) => <CustomHeading level={3} {...props} {...commonProps} />);
  const h4 = useMemoizedFn((props) => <CustomHeading level={4} {...props} {...commonProps} />);
  const h5 = useMemoizedFn((props) => <CustomHeading level={5} {...props} {...commonProps} />);
  const h6 = useMemoizedFn((props) => <CustomHeading level={6} {...props} {...commonProps} />);
  const customComponents = useMemoizedFn(() => AppMarkdownCustomComponents(commonProps));

  const memoizedComponents: Partial<Components> = useMemo(() => {
    return {
      //custom components
      ...customComponents(),
      //common components,
      text,
      code,
      heading,
      listItem,
      blockquote,
      span,
      p,
      h1,
      h2,
      h3,
      h4,
      h5,
      h6,
      li
    };
  }, []);

  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      skipHtml={true}
      components={memoizedComponents}
      rehypePlugins={[rehypeRaw]} //rehypeSanitize we will assume that the markdown is safe? If we use it we cant do web components
      className={cx(styles.container, 'space-y-2.5', className)}>
      {currentMarkdown}
    </ReactMarkdown>
  );
};

export const AppMarkdown = memo(_AppMarkdown);

const useStyles = createStyles(({ token, css }) => {
  return {
    container: css`
      line-height: 20px;
      font-size: ${token.fontSize}px;
      h1 {
        font-size: ${token.fontSizeHeading1}px;
      }
      h2 {
        font-size: ${token.fontSizeHeading2}px;
      }
      h3 {
        font-size: ${token.fontSizeHeading3}px;
      }
      h4 {
        font-size: ${token.fontSizeHeading4}px;
      }
      h5 {
        font-size: ${token.fontSizeHeading5}px;
      }
      p {
        font-size: ${token.fontSize}px;
      }
      h1,
      h2,
      h3,
      h4,
      h5 {
        width: fit-content;
        display: flex;
        align-items: center;
        gap: 4px;
        margin-top: 20px !important;
        &:first-child {
          margin-top: 0 !important;
        }
      }

      table {
        border: 0.5px solid ${token.colorBorder};
        border-radius: ${token.borderRadius}px;

        th {
          min-width: 30px;
          border-bottom: 0.5px solid ${token.colorBorder};
          padding: 6px;
          border-right: 0.5px solid ${token.colorBorder};
          &:last-child {
            border-right: none;
          }
        }
        td {
          border-right: 0.5px solid ${token.colorBorder};
          &:last-child {
            border-right: none;
          }
          padding: 6px;
        }
        tr {
          border-bottom: 0.5px solid ${token.colorBorder};
          &:last-child {
            border-bottom: none;
          }
        }
      }

      ul,
      ol,
      dl {
        margin-left: ${token.margin}px;
        list-style: revert;
      }

      a {
        color: ${token.colorPrimary};
      }
    `
  };
});

export default AppMarkdown;
