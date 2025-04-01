import type { ConfigProviderProps, ThemeConfig } from 'antd';
import tailwindConfig from '../../tailwind.config';
import { createStyles } from 'antd-style';
import { AppMaterialIcons } from '@/components/icons/AppMaterialIcons';

const screens = tailwindConfig.theme.screens;

export const busterAppStyleConfig: ThemeConfig = {
  token: {
    lineWidth: 0.5,
    colorPrimary: '#7C3AED',
    colorSuccess: '#34A32D',
    controlItemBgActive: '#F3F3F3',
    controlItemBgHover: '#F8F8F8',
    controlItemBgActiveHover: 'rgb(232 232 232)',
    colorBgTextActive: 'rgb(0, 0, 0, 0.125)',
    colorTextBase: 'rgb(0, 0, 0, 0.88)',
    colorTextDescription: '#575859',
    colorTextSecondary: '#575859',
    colorTextPlaceholder: '#969597',
    colorTextTertiary: '#969597',
    colorError: '#C4242A',
    colorLink: 'rgb(0, 0, 0, 0.88)',
    colorLinkHover: 'rgb(0, 0, 0, 0.85)',
    colorBorder: '#E0E0E0',
    colorBgContainerDisabled: '#FCFCFC',
    borderRadiusLG: 4,
    borderRadiusSM: 4,
    borderRadiusXS: 4,
    colorTextDisabled: '#969597',
    colorIcon: '#575859',
    colorSplit: '#E0E0E0',
    colorBgSpotlight: 'rgb(255, 255, 255)',
    colorBgMask: 'rgba(0, 0, 0, 0.125)',
    screenXS: parseInt(screens.xs),
    screenSM: parseInt(screens.sm),
    screenMD: parseInt(screens.md),
    screenLG: parseInt(screens.lg),
    screenXL: parseInt(screens.xl),
    screenXXL: parseInt(screens.xxl),
    borderRadius: 4,
    controlHeightSM: 24,
    controlHeight: 28,
    controlHeightLG: 36,
    fontWeightStrong: 500,
    fontSizeSM: 10,
    fontSize: 13,
    fontSizeLG: 15,
    fontSizeXL: 18,
    fontSizeHeading1: 24,
    fontSizeHeading2: 20,
    fontSizeHeading3: 18,
    fontSizeHeading4: 15,
    fontSizeHeading5: 13,
    lineHeight: 1.4,
    lineHeightHeading1: 1.3,
    lineHeightHeading2: 1.3,
    lineHeightHeading3: 1.3,
    lineHeightHeading4: 1.3,
    lineHeightHeading5: 1.3,
    fontFamily: 'Roobert_Pro'
  },
  components: {
    Typography: {
      titleMarginBottom: 0,
      titleMarginTop: 0,
      fontWeightStrong: 400
    },
    Menu: {
      itemSelectedColor: 'rgb(0, 0, 0)',
      itemSelectedBg: '#E6E6E6',
      itemHoverBg: '#EFEEEE',
      itemColor: 'rgb(0, 0, 0)',
      itemBg: 'transparent',
      fontSize: 13,
      groupTitleFontSize: 13,
      activeBarBorderWidth: 0,
      iconSize: 16,
      iconMarginInlineEnd: 10,
      itemHeight: 28,
      itemMarginBlock: 1,
      itemMarginInline: 0,
      margin: 6,
      padding: 4
    },
    Modal: {
      fontSizeIcon: 15,
      titleFontSize: 15,
      fontWeightStrong: 400,
      fontSizeLG: 15
    },
    Skeleton: {
      controlHeightXS: 12,
      controlHeightSM: 14,
      paragraphMarginTop: 8,
      marginSM: 0
    },
    Tooltip: {
      controlHeight: 10,
      fontSize: 13,
      paddingSM: 6,
      colorTextLightSolid: 'rgb(0, 0, 0, 0.88)',
      colorText: 'rgb(255, 255, 255, 0.88)', //0px 0px 0px 1px
      boxShadowSecondary: `0px 1px 0.5px 0px rgba(0, 0, 0, 0.05)` //.monaco-editor-overlaymessage
    },
    Popover: {
      padding: 0,
      paddingSM: 0,
      paddingContentHorizontal: 0,
      paddingLG: 0,
      paddingMD: 0,
      paddingXL: 0,
      paddingContentHorizontalLG: 0,
      paddingContentHorizontalSM: 0,
      paddingContentVertical: 0,
      paddingContentVerticalLG: 0,
      paddingContentVerticalSM: 0,
      paddingXS: 0,
      paddingXXS: 0,
      boxShadowSecondary: `0px 7px 10px 0px rgb(0 0 0 / 7%)` //SAME as Tooltip
    },
    Dropdown: {
      colorPrimary: 'black',
      colorTextDisabled: '#969597',
      boxShadowSecondary: `0px 7px 10px 0px rgb(0 0 0 / 7%)` //SAME as Tooltip
    },
    Notification: {
      width: 300,
      paddingMD: 12,
      fontSizeLG: 14,
      paddingContentHorizontalLG: 12,
      boxShadow: '0px 7px 10px 0px rgb(0 0 0 / 7%)',
      boxShadowSecondary: `0px 7px 10px 0px rgb(0 0 0 / 7%)` //SAME as Tooltip
    },
    Message: {
      boxShadow: '0px 7px 10px 0px rgb(0 0 0 / 7%)',
      boxShadowSecondary: `0px 7px 10px 0px rgb(0 0 0 / 7%)` //SAME as Tooltip
    },
    Input: {
      inputFontSizeLG: 15,
      addonBg: 'transparent',
      paddingInline: 8
    },
    Switch: {
      colorPrimary: '#7C3AED',
      trackMinWidth: 32
    },
    Button: {
      margin: 0,
      padding: 0,
      paddingInline: 8,
      marginXS: 4,
      defaultShadow: 'transparent',
      controlHeight: 24,
      //PRIMARY BUTTON
      colorPrimaryBg: 'rgb(0, 0, 0, 0.88)',
      colorPrimaryBgHover: '#A26CFF',
      colorIcon: '#575859',
      //DEFAULT BUTTON
      defaultColor: 'rgb(0, 0, 0, 0.88)',
      defaultHoverColor: '#575859',
      defaultHoverBorderColor: '#E0E0E0',
      defaultHoverBg: '#f8f8f8',
      defaultActiveColor: '#575859',
      defaultActiveBorderColor: '#575859',
      defaultActiveBg: '#F3F3F3',
      //TEXT BUTTON
      textHoverBg: 'rgba(0, 0, 0, 0.05)',
      textTextColor: '#575859',
      textTextHoverColor: 'rgb(0, 0, 0, 0.88)',
      //ALL
      colorTextBase: 'rgb(0, 0, 0, 0.88)',
      colorBgTextActive: 'rgba(0, 0, 0, 0.075)',
      colorBgContainerDisabled: '#F3F3F3',
      colorTextDisabled: '#969597',
      borderColorDisabled: '#E0E0E0'
    },
    Breadcrumb: {
      separatorMargin: 4,
      colorText: '#575859',
      lastItemColor: 'rgb(0, 0, 0, 0.88)',
      itemColor: '#575859',
      colorBgTextHover: 'rgba(0, 0, 0, 0)',
      linkColor: 'rgba(0, 0, 0, 1)'
    },
    Select: {
      fontFamily:
        '--apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Helvetica Neue, Arial, sans-serif',
      controlPaddingHorizontal: 8
    },
    Tabs: {
      horizontalItemPadding: '10px 0px',
      titleFontSize: 15,
      horizontalItemGutter: 20,
      itemActiveColor: 'rgb(0, 0, 0)',
      itemColor: 'rgba(123, 123, 123, 0.88)',
      inkBarColor: 'rgb(0, 0, 0)',
      itemHoverColor: 'rgb(0, 0, 0)',
      itemSelectedColor: 'rgb(0, 0, 0)'
    },
    Segmented: {
      controlPaddingHorizontal: 8.5,
      itemColor: '#575859',
      boxShadowTertiary: 'transparent',
      trackPadding: 0,
      fontSize: 13, //override sep 3 2024
      itemHoverBg: 'transparent',
      itemActiveBg: 'transparent'
    },
    Layout: {
      headerBg: '#FCFCFC',
      bodyBg: '#f8f8f8',
      headerHeight: 40,
      headerPadding: '0 24px 0 30px',
      siderBg: 'transparent'
    },
    Divider: {
      marginLG: 0
    },
    Table: {
      borderRadius: 0,
      borderRadiusLG: 0,
      colorFillAlter: 'transparent',
      colorFillSecondary: 'transparent',
      colorBgContainer: '#fff',
      headerBorderRadius: 0,
      lineHeight: 1,
      cellPaddingBlock: 16,
      headerBg: '#fff',
      rowHoverBg: '#F8F8F8'
    },
    Steps: {
      fontSize: 32,
      iconTop: -1.105,
      dotSize: 7,
      fontSizeLG: 12
    },
    Tag: {
      colorBorder: '#E0E0E0',
      colorText: '#575859',
      fontSizeSM: 9,
      defaultBg: '#F3F3F3'
    },
    Card: {
      boxShadow: 'transparent',
      //@ts-ignore
      boxShadowCard: 'transparent',
      boxShadowTertiary: 'transparent',
      colorBgContainer: '#FFF',
      colorBorderSecondary: '#E0E0E0',
      lineWidth: 0.5,
      fontWeightStrong: 400,
      borderRadiusLG: 6,
      borderRadiusSM: 6,
      borderRadiusXS: 6
    },
    Slider: {
      trackBg: '#E0E0E0'
    },
    Collapse: {
      headerBg: 'transparent',
      fontSize: 12
    },
    Avatar: {
      groupOverlapping: -4,
      groupSpace: 4,
      groupBorderColor: 'transparent'
    },
    Checkbox: {
      controlInteractiveSize: 14
    },
    Form: {
      itemMarginBottom: 0
    }
  }
};

export const busterAppStyleConfigDark: ThemeConfig = {
  token: {
    ...busterAppStyleConfig.token,
    controlItemBgActive: '#2C2C2C',
    controlItemBgHover: '#3A3A3A',
    controlItemBgActiveHover: 'rgb(70, 70, 70)',
    colorBgTextActive: 'rgba(255, 255, 255, 0.125)',
    colorTextBase: 'rgba(255, 255, 255, 0.85)',
    colorTextDescription: '#a1a1a1',
    colorTextSecondary: '#a1a1a1',
    colorTextPlaceholder: '#6B6B6B',
    colorTextTertiary: '#6B6B6B',
    colorLink: 'rgba(255, 255, 255, 0.85)',
    colorLinkHover: 'rgba(255, 255, 255, 0.95)',
    colorBorder: '#434343',
    colorBgContainerDisabled: '#1F1F1F',
    colorTextDisabled: '#6B6B6B',
    colorIcon: '#a1a1a1',
    colorSplit: '#434343',
    colorBgSpotlight: 'rgb(30, 30, 30)',
    colorBgMask: 'rgba(0, 0, 0, 0.45)'
  },
  components: {
    ...busterAppStyleConfig.components,
    Menu: {
      ...busterAppStyleConfig.components?.Menu,
      itemSelectedColor: 'rgb(255, 255, 255)',
      itemSelectedBg: '#3A3A3A',
      itemHoverBg: '#2C2C2C',
      itemColor: 'rgb(255, 255, 255)'
    },
    Tooltip: {
      ...busterAppStyleConfig.components?.Tooltip,
      colorTextLightSolid: 'rgb(255, 255, 255, 0.88)',
      colorText: 'rgb(0, 0, 0, 0.88)', //0px 0px 0px 1px
      boxShadowSecondary: `0px 1px 0.5px 0px rgba(255, 255, 255, 0.05)` //.monaco-editor-overlaymessage
    },
    Popover: {
      ...busterAppStyleConfig.components?.Popover,
      boxShadowSecondary: `0px 1px 0.5px 0px rgba(255, 255, 255, 0.05)` //SAME as Tooltip
    },
    Dropdown: {
      colorPrimary: 'white',
      colorTextDisabled: '#6B6B6B',
      boxShadowSecondary: `0px 1px 0.5px 0px rgba(255, 255, 255, 0.05)` //SAME as Tooltip
    },
    Notification: {
      ...busterAppStyleConfig.components?.Notification,
      boxShadow: '0px 7px 10px 0px rgba(255, 255, 255, 0.07)',
      boxShadowSecondary: `0px 1px 0.5px 0px rgba(255, 255, 255, 0.05)` //SAME as Tooltip
    },
    Message: {
      boxShadow: '0px 7px 10px 0px rgba(255, 255, 255, 0.07)',
      boxShadowSecondary: `0px 1px 0.5px 0px rgba(255, 255, 255, 0.05)` //SAME as Tooltip
    },

    Button: {
      ...busterAppStyleConfig.components?.Button,
      //PRIMARY BUTTON
      colorPrimaryBg: '#7C3AED',
      colorPrimaryBgHover: '#9B5EFF',
      colorIcon: '#a1a1a1',
      //DEFAULT BUTTON
      defaultColor: 'rgba(255, 255, 255, 0.85)',
      defaultBg: '#000',
      defaultHoverColor: '#fff',
      defaultHoverBorderColor: '#434343',
      defaultHoverBg: '#2C2C2C',
      defaultActiveColor: '#000',
      defaultActiveBorderColor: '#575859',
      defaultActiveBg: '#3A3A3A',
      //TEXT BUTTON
      textHoverBg: 'rgba(255, 255, 255, 0.08)',
      colorTextBase: 'rgba(255, 255, 255, 0.85)',
      colorBgTextActive: 'rgba(255, 255, 255, 0.12)',
      //ALL
      colorBgContainerDisabled: '#2C2C2C',
      colorTextDisabled: '#6B6B6B',
      borderColorDisabled: '#434343'
    },
    Breadcrumb: {
      ...busterAppStyleConfig.components?.Breadcrumb,
      colorText: '#a1a1a1',
      lastItemColor: 'rgba(255, 255, 255, 0.85)',
      itemColor: '#a1a1a1',
      colorBgTextHover: 'rgba(255, 255, 255)',
      linkColor: 'rgba(255, 255, 255)'
    },
    Tabs: {
      ...busterAppStyleConfig.components?.Tabs,
      itemActiveColor: 'rgb(255, 255, 255)',
      itemColor: 'rgba(170, 170, 170, 0.88)',
      inkBarColor: 'rgb(255, 255, 255)',
      itemHoverColor: 'rgb(200, 200, 200)',
      itemSelectedColor: 'rgb(255, 255, 255)'
    },
    Segmented: {
      ...busterAppStyleConfig.components?.Segmented,
      itemColor: '#a1a1a1'
    },
    Layout: {
      ...busterAppStyleConfig.components?.Layout,
      headerBg: '#1f1f1f',
      bodyBg: '#141414'
    },

    Table: {
      ...busterAppStyleConfig.components?.Table,
      colorBgContainer: '#000',
      headerBg: '#000'
    },
    Tag: {
      ...busterAppStyleConfig.components?.Tag,
      colorBorder: '#434343',
      colorText: '#a1a1a1',
      defaultBg: '#2C2C2C'
    },
    Card: {
      ...busterAppStyleConfig.components?.Card,
      colorBgContainer: '#000',
      colorBorderSecondary: '#434343'
    },
    Slider: {
      trackBg: '#434343'
    }
  }
};

const useComponentStyles = createStyles(({ css, prefixCls }) => {
  return {
    modalContent: css``,
    modalFooter: css`
      border-top: 0.5px solid ${busterAppStyleConfig.token?.colorBorder} !important;
      display: flex;
      align-items: center;
      justify-content: flex-end;

      .dark & {
        border-top: 0.5px solid ${busterAppStyleConfigDark.token?.colorBorder} !important;
      }
    `,
    breadcrumb: css`
      background-color: inherit; //there was a bug where we had to put this to make overflow work? (2 months later looking at this comment it seems crazy...)
      &.busterv2-breadcrumb {
        overflow: hidden;
        ol {
          height: 100%;
          display: flex;
          align-items: center;
          flex-wrap: nowrap;
          overflow: hidden;
        }
        .busterv2-breadcrumb-separator {
          display: flex;
          align-items: center;
        }
        .busterv2-breadcrumb-link {
          display: block; //make sure it can truncate?
          align-items: center;
          overflow: hidden;
          //  height: 100%; //make sure it centers vertically
          text-overflow: ellipsis;
          .busterv2-typography,
          a {
            height: 100%;
            display: flex;
            align-items: center;
          }
        }
        li:last-child {
          overflow: hidden;
          text-overflow: ellipsis;
          text-wrap: nowrap;
          height: 100%;
        }
        li:first-child {
          .busterv2-breadcrumb-link {
            a {
              padding: 0;
              margin-inline: 0;
            }
          }
        }
        li {
          display: flex;
          align-items: center;
        }

        li:not(:last-child) {
          .busterv2-breadcrumb-link {
            color: ${busterAppStyleConfig.token?.colorTextSecondary};
            a {
              color: ${busterAppStyleConfig.token?.colorTextSecondary};
            }
          }
        }
      }
    `,
    segmented: css`
      .busterv2-segmented-item-icon {
        display: flex;
        color: ${busterAppStyleConfig.token?.colorIcon};
      }

      .busterv2-segmented-thumb,
      .busterv2-segmented-item-selected .busterv2-segmented-item-label {
        box-shadow: inset 0 0 0 0.5px ${busterAppStyleConfig.token?.colorBorder};
        border-radius: ${busterAppStyleConfig.token?.borderRadius}px;
      }
      &.no-border {
        .busterv2-segmented-thumb,
        .busterv2-segmented-item-selected .busterv2-segmented-item-label {
          box-shadow: inset 0 0 0 0px ${busterAppStyleConfig.token?.colorBorder};
        }
      }

      .busterv2-segmented-item-label {
        display: flex;
        align-items: center;
        justify-content: center;
        a {
          display: flex;
          align-items: center;
        }
      }

      box-shadow: inset 0px 0px 0px 0.5px ${busterAppStyleConfig.token?.colorBorder};

      .dark & {
        box-shadow: inset 0px 0px 0px 0.5px ${busterAppStyleConfigDark.token?.colorBorder};

        .busterv2-segmented-thumb,
        .busterv2-segmented-item-selected {
          border: 0.5px solid ${busterAppStyleConfigDark.token?.colorBorder} !important;
        }
      }
    `,
    button: css`
      &.busterv2-btn-text,
      &.busterv2-btn-default {
        &:not(.busterv2-btn-icon-only) {
          .busterv2-btn-icon {
            margin-left: -1px;
          }
        }

        .busterv2-btn-icon {
          display: flex;
          color: ${busterAppStyleConfig.token?.colorIcon};
        }

        &.busterv2-btn-dangerous {
          .busterv2-btn-icon {
            color: inherit;
          }
        }

        .dark & {
          .busterv2-btn-icon {
            color: ${busterAppStyleConfigDark.token?.colorIcon};
          }
        }
      }

      &:disabled {
        &.busterv2-btn-default,
        &.busterv2-btn-text {
          &.busterv2-btn-icon-only {
            .busterv2-btn-icon {
              color: ${busterAppStyleConfig.token?.colorTextDisabled};
            }
          }
        }
      }
    `,
    dropdown: css`
      .busterv2-dropdown-menu {
        min-width: 204px;
      }

      .busterv2-dropdown-menu-submenu,
      .busterv2-dropdown-menu-item {
        padding-top: 0px !important;
        padding-bottom: 0px !important;
        min-height: 28px;
        position: relative;
      }

      .busterv2-dropdown-menu-submenu-title {
        display: flex;
        align-items: center;
      }

      .busterv2-dropdown-menu-item:not(.busterv2-dropdown-menu-item-disabled) {
        .busterv2-dropdown-menu-item-icon {
          color: ${busterAppStyleConfig.token?.colorIcon};
        }
      }

      .dark & {
        .busterv2-dropdown-menu-item-icon {
          color: ${busterAppStyleConfigDark.token?.colorIcon};
        }
      }
    `,
    input: css`
      .busterv2-input-prefix {
        color: ${busterAppStyleConfig.token?.colorIcon};
      }

      .dark & {
        .busterv2-input-prefix {
          color: ${busterAppStyleConfigDark.token?.colorIcon};
        }
      }
    `,
    menu: css`
      .busterv2-menu-submenu,
      .busterv2-menu-item {
        display: flex;

        .busterv2-menu-item-icon {
          color: ${busterAppStyleConfig.token?.colorIcon} !important;
        }

        &.busterv2-menu-item-disabled {
          .busterv2-menu-item-icon {
            color: ${busterAppStyleConfig.token?.colorTextDisabled} !important;
          }
        }
      }

      .busterv2-menu-submenu-title {
        display: flex;
        align-items: center;
      }

      .dark & {
        .busterv2-menu-item {
          .busterv2-menu-item-icon {
            color: ${busterAppStyleConfigDark.token?.colorIcon} !important;
          }
        }
      }
    `
  };
});

export const useBusterAppComponentConfig = (): Pick<
  ConfigProviderProps,
  'modal' | 'breadcrumb' | 'segmented' | 'button' | 'dropdown' | 'input' | 'menu' | 'select'
> => {
  const { styles, cx } = useComponentStyles();

  return {
    modal: {
      classNames: {
        body: cx('!px-8 !pt-4 !pb-6'),
        header: '!mb-0 !px-8 !pt-6',
        content: cx('!p-0', styles.modalContent),
        footer: cx('!mt-0 !px-3 !py-3', styles.modalFooter)
      }
    },
    breadcrumb: {
      className: cx(styles.breadcrumb)
    },
    segmented: {
      className: cx(styles.segmented)
    },
    button: {
      className: cx(styles.button)
    },
    dropdown: {
      className: cx(styles.dropdown)
    },
    input: {
      className: cx(styles.input)
    },
    menu: {
      className: cx(styles.menu),
      expandIcon: (d) => {
        return (
          <AppMaterialIcons
            style={{
              transform: d.isOpen ? 'rotate(90deg)' : 'rotate(0deg)',
              transition: 'transform 0.3s',
              position: 'absolute',
              insetInlineEnd: 2,
              color: '#575859'
            }}
            icon="chevron_right"
          />
        );
      }
    }
  };
};
