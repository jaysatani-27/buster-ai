import { createStyles } from 'antd-style';

export const useEditTitleStyles = createStyles(({ token, css }) => {
  return {
    editTitle: css`
      overflow: hidden;
      .busterv2-typography-edit-content {
        inset-inline-start: 0 !important;
        margin-top: 0 !important;
        margin-bottom: 0 !important;
      }
      .busterv2-input {
        padding: 0 !important;
        padding-right: 0px !important; //I changed this from 25px? to 0
        border: none !important;
      }
    `,
    editText: css``,
    editInput: css`
      padding: 0 !important;
      padding-right: 25px !important;
      border: none !important;
    `
  };
});
