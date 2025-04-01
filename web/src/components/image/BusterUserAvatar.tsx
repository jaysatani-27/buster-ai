import React, { useMemo } from 'react';
import { Avatar, AvatarProps } from 'antd';
import { getFirstTwoCapitalizedLetters } from '@/utils/text';
import { AppTooltip } from '../tooltip';
import type { GroupProps } from 'antd/es/avatar';
import { useBusterStylesContext } from '@/context/BusterStyles/BusterStyles';
import BusterIconWhite from '@/assets/png/buster_icon_small_white.png';
import BusterIconBlack from '@/assets/png/buster_icon_small_black.png';
import { createStyles } from 'antd-style';
export interface BusterUserAvatarProps extends AvatarProps {
  color?: string;
  image?: string;
  name?: string | null;
  style?: React.CSSProperties;
  useToolTip?: boolean;
}

export const BusterUserAvatar = React.memo(
  ({ useToolTip = true, ...props }: BusterUserAvatarProps) => {
    const { size = 38, ...restProps } = props;
    if (!props.name && !props.image) return <BusterAvatar {...restProps} size={size as number} />;

    const firstAndLastInitial = createNameLetters(props.name, props.image);

    return (
      <AppTooltip
        title={useToolTip ? props.name || '' : ''}
        mouseEnterDelay={0.35}
        performant={useToolTip}>
        <Avatar
          {...props}
          className={`${props.className || ''} flex ${props.image ? '!bg-transparent' : ''}`}
          size={size}
          style={{
            minWidth: size as number,
            minHeight: size as number
          }}
          src={createAvatarImage(props.image, firstAndLastInitial, props.name)}>
          {firstAndLastInitial}
        </Avatar>
      </AppTooltip>
    );
  }
);
BusterUserAvatar.displayName = 'BusterUserAvatar';

const useStyles = createStyles(({ css, token, isDarkMode }) => {
  return {
    groupAvatar: css`
      .busterv2-avatar {
        font-size: ${token.fontSizeSM} !important;
      }
    `,
    avatar: {
      background: isDarkMode ? token.colorBgSpotlight : token.colorFillContentHover
    }
  };
});

export const BusterUserAvatarGroup: React.FC<
  {
    avatars: BusterUserAvatarProps[];
  } & GroupProps
> = React.memo(({ avatars, ...props }) => {
  const { styles, cx } = useStyles();

  const renderedAvatars = useMemo(
    () =>
      avatars.map((avatar, index) => (
        <BusterUserAvatar size={props.size} key={index} {...avatar} />
      )),
    [avatars, props.size]
  );

  return (
    <>
      <Avatar.Group {...props} className={cx(styles.groupAvatar, props.className)}>
        {renderedAvatars}
      </Avatar.Group>
    </>
  );
});
BusterUserAvatarGroup.displayName = 'BusterUserAvatarGroup';

const createNameLetters = (name?: string | null, image?: string | null | React.ReactNode) => {
  if (name && !image) {
    const firstTwoLetters = getFirstTwoCapitalizedLetters(name);
    if (firstTwoLetters.length == 2) return firstTwoLetters;

    //Get First Name Initial
    const _name = name.split(' ') as [string, string];
    const returnName = `${_name[0][0]}`.toUpperCase().replace('@', '');

    return returnName;
  }
  return '';
};

const createAvatarImage = (
  image: string | null | React.ReactNode,
  initialNames?: null | string,
  fullname?: string | null
) => {
  if (image) return image;
  if (!initialNames) return;
  //USER AVATARS

  // const removeHash = hex.replace('#', '');
  // const baseUrl = `https://ui-avatars.com/api/?background=${removeHash}&color=fff&name=${name}`;
  // return baseUrl;

  //VERCEL
  // const baseUrl = `https://avatar.vercel.sh/${fullname}.svg?text=${initialNames}`;
  // return baseUrl;

  //DICE BEAR INDENTICON
  const baseUrl = `https://api.dicebear.com/9.x/initials/svg?seed=${fullname || initialNames}`;
  return baseUrl;
};

export const BusterAvatar: React.FC<{
  size?: number;
  shape?: 'circle' | 'square';
}> = React.memo(({ size = 24, shape = 'circle' }) => {
  const { styles, cx } = useStyles();

  const memoizedStyles = useMemo(
    () => ({
      minWidth: size,
      minHeight: size
    }),
    [size]
  );

  return (
    <AppTooltip title={'Buster'}>
      <Avatar
        size={size}
        shape={shape}
        icon={<BusterImage />}
        className={cx(styles.avatar)}
        style={memoizedStyles}
      />
    </AppTooltip>
  );
});
BusterAvatar.displayName = 'BusterAvatar';

const BusterImage: React.FC = () => {
  const isDarkMode = useBusterStylesContext((s) => s.isDarkMode);
  const image = isDarkMode ? BusterIconBlack.src : BusterIconWhite.src;

  return (
    <div
      className={`flex h-full w-full items-center justify-center ${
        isDarkMode ? 'bg-gray-900' : 'bg-white'
      }`}>
      <img
        className="flex items-center justify-center"
        style={{
          height: '100%',
          width: 'auto',
          objectFit: 'contain'
        }}
        src={image}
        alt="Company Logo"
      />
    </div>
  );
};
