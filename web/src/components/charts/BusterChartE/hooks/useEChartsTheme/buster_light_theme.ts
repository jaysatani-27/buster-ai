import { determineFontColorContrast } from '@/utils';
import { DEFAULT_CHART_THEME } from '../../../configColors';
import { busterAppStyleConfig } from '@/styles/busterAntDStyleConfig';

const token = busterAppStyleConfig.token!;

const gridColor = '#E5E7EB';

export const defaultLabelFormat = {
  borderRadius: 4,
  padding: [3, 6],
  // color: token.colorTextSecondary,
  backgroundColor: token.colorBgContainerDisabled,
  borderWidth: 0.5,
  borderColor: token.colorBorder,
  fontSize: 10,
  rich: {
    light: {
      color: token.colorTextSecondary,
      fontSize: 10
    },
    dark: {
      color: '#F5F5F5',
      fontSize: 10
    }
  }
};

export const labelContrastFormatter = (value: string, color: string) => {
  const contrastColor = determineFontColorContrast(color);
  const variable = contrastColor === '#000000' ? 'light' : 'dark';
  return `{${variable}|${value}}`;
};

export const busterLightTheme = {
  version: 1,
  themeName: 'buster',
  theme: {
    color: DEFAULT_CHART_THEME,
    backgroundColor: 'transparent',
    textStyle: {
      fontFamily: token.fontFamily
    },
    line: {
      itemStyle: {
        borderWidth: 2
      },
      lineStyle: {
        width: 2,
        type: 'solid'
      },
      symbolSize: 8,
      symbol: 'emptyCircle',
      showAllSymbol: true,
      smooth: false,
      triggerLineEvent: false,
      emphasis: {
        focus: 'self' //self, series, none,
      },
      label: {
        show: false,
        position: 'top'
      }
    },
    bar: {
      clip: true,
      itemStyle: {
        barBorderWidth: 0
      },
      emphasis: {
        focus: 'none', //self, series, none,
        disabled: true
      },
      label: {
        overflow: 'truncate',
        show: false,
        position: 'insideTop',
        align: 'center',
        verticalAlign: 'top',
        backgroundColor: 'inherit', //inherit makes it the same as the chart background
        borderWidth: 0,
        minMargin: 6,
        padding: [1.5, 3]
      }
    },
    pie: {
      avoidLabelOverlap: true,
      itemStyle: {
        borderRadius: 5,
        borderColor: '#ffffff',
        borderWidth: 2
      },
      label: {
        backgroundColor: 'inherit', //inherit makes it the same as the chart background
        borderWidth: 0,
        alignTo: 'none', //none, labelLine, edge
        //@ts-ignore
        color: null
      },
      labelLine: {
        show: true
      }
    },
    scatter: {
      itemStyle: {
        borderWidth: 0,
        borderColor: '#ccc'
      },
      emphasis: {
        scale: 1.35, // Scale factor when hovering
        focus: 'series' //self, series, none
      }
    },
    categoryAxis: {
      axisPointer: {
        show: true,
        type: 'shadow',
        triggerEmphasis: false
      },
      axisLine: {
        show: true,
        lineStyle: {
          color: gridColor
        }
      },
      axisTick: {
        show: false,
        lineStyle: {
          color: gridColor
        },
        alignWithLabel: true,
        overflow: 'truncate'
      },
      axisLabel: {
        show: true,
        color: token.colorTextTertiary,
        hideOverlap: true
      },
      nameTextStyle: {
        align: 'center',
        color: token.colorTextSecondary,
        verticalAlign: 'top'
      },
      nameGap: 26,
      nameLocation: 'middle',
      splitLine: {
        show: true
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: ['rgba(250,250,250,0.15)', 'rgba(200,200,200,0.12)']
        }
      }
    },
    valueAxis: {
      alignTicks: true,
      axisLine: {
        show: false,
        lineStyle: {
          color: gridColor
        }
      },
      axisTick: {
        show: false,
        lineStyle: {
          color: '#333'
        }
      },
      splitLine: {
        show: true
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: ['rgba(250,250,250,0.05)', 'rgba(200,200,200,0.02)']
        }
      },
      nameTextStyle: {
        color: token.colorTextSecondary
      },
      nameGap: 46,
      nameLocation: 'middle'
    },
    logAxis: {
      alignTicks: true
    },
    timeAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: token.colorBorder
        }
      },
      axisTick: {
        show: false,
        lineStyle: {
          color: gridColor
        }
      },
      axisLabel: {
        show: true,
        hideOverlap: true,
        color: token.colorTextTertiary,
        formatter: {
          year: '{yearStyle|{yyyy}}',
          month: '{monthStyle|{MMM}}',
          day: '{dayStyle|{dd}}',
          hour: '{hourStyle|{HH}}',
          minute: '{minuteStyle|{mm}}',
          second: '{secondStyle|{ss}}',
          millisecond: '{millisecondStyle|{SSS}}'
        },
        rich: {
          yearStyle: {
            fontWeight: 'normal',
            color: token.colorTextSecondary
          },
          monthStyle: {
            color: token.colorTextTertiary
          },
          dayStyle: {
            color: token.colorTextTertiary
          },
          hourStyle: {
            color: token.colorTextTertiary
          },
          minuteStyle: {
            color: token.colorTextTertiary
          },
          secondStyle: {
            color: token.colorTextTertiary
          },
          millisecondStyle: {
            color: token.colorTextTertiary
          }
        }
      },
      splitLine: {
        show: true
      },
      splitArea: {
        show: false
      }
    },
    legend: {
      show: false //this is overriden by custom legend
    },
    tooltip: {
      trigger: 'axis',
      borderRadius: token.borderRadius,
      borderColor: token.colorBorder,
      borderWidth: 0.5,
      axisPointer: {
        lineStyle: {
          color: '#AAAAAA',
          width: 1
        },
        crossStyle: {
          color: '#AAAAAA',
          width: 1
        },
        shadowStyle: {
          color: 'rgba(0, 0, 0, 0.035)' // Soft grey shadow
        },
        label: {
          color: token.colorTextSecondary,
          backgroundColor: token.colorBgContainer
        }
      },
      className: 'buster-tooltip !p-0'
    },
    timeline: {},
    visualMap: {
      color: ['#516b91', '#59c4e6', '#a5e7f0']
    },
    dataZoom: {
      backgroundColor: 'rgba(0,0,0,0)',
      dataBackgroundColor: 'rgba(255,255,255,0.3)',
      fillerColor: 'rgba(167,183,204,0.4)',
      handleColor: '#a7b7cc',
      handleSize: '100%',
      textStyle: {
        color: '#333'
      }
    },
    markPoint: {
      label: {
        ...defaultLabelFormat
      },
      emphasis: {
        label: {
          color: token.colorTextSecondary,
          backgroundColor: token.colorBgMask
        }
      }
    },
    markLine: {
      silent: true,
      animation: false,
      symbol: ['none', 'none'],
      label: {
        position: 'middle', //https://echarts.apache.org/en/option.html#series-bar.markLine.label.position
        color: token.colorTextBase,
        backgroundColor: token.colorBgContainerDisabled,
        padding: [3, 6],
        borderRadius: token.borderRadius,
        borderWidth: 0.5,
        borderColor: token.colorBorder
      },
      lineStyle: {
        color: '#000000',
        width: 2,
        opacity: 0.94
      }
    },
    endLabel: {
      ...defaultLabelFormat,
      verticalAlign: 'bottom',
      align: 'right',
      offset: [5, 0],
      valueAnimation: false
    },
    label: defaultLabelFormat,
    labelLayout: {
      hideOverlap: true
    },
    grid: {
      left: '2%',
      right: '2%',
      bottom: '2%',
      top: '5%',
      containLabel: true
    },
    axisLabel: {
      show: true,
      hideOverlap: true,
      color: '#999999'
    },
    splitLine: {
      lineStyle: {
        color: gridColor
      }
    }
  }
};

export default busterLightTheme;
