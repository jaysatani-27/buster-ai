import { AppMaterialIcons } from '@/components';
import React, { useEffect, useMemo } from 'react';
import { AppPopover, Text } from '@/components';

export const PolicyCheck: React.FC<{
  password: string;
  show: boolean;
  onCheckChange?: (value: boolean) => void;
  children?: React.ReactNode;
  placement?: 'top' | 'right' | 'bottom' | 'left';
}> = ({ password, show, onCheckChange, children, placement = 'left' }) => {
  const items = useMemo(() => {
    const containsNumber = /\d/;
    const containsSpecialChar = /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]+/;
    const containsUppercase = /[A-Z]/;
    const containsLowercase = /[a-z]/;

    const numberCheck = containsNumber.test(password);
    const specialCharCheck = containsSpecialChar.test(password);
    const uppercaseCheck = containsUppercase.test(password);
    const lowercaseCheck = containsLowercase.test(password);
    const passwordLengthCheck = password.length >= 8;

    const passwordGood = {
      numberCheck,
      specialCharCheck,
      uppercaseCheck,
      lowercaseCheck,
      passwordLengthCheck
    };

    const items = [
      {
        text: 'Contains a number',
        check: passwordGood.numberCheck
      },
      {
        text: 'Contains a special character',
        check: passwordGood.specialCharCheck
      },
      {
        text: 'Contains an uppercase letter',
        check: passwordGood.uppercaseCheck
      },
      {
        text: 'Contains a lowercase letter',
        check: passwordGood.lowercaseCheck
      },
      {
        text: 'Is at least 8 characters long',
        check: passwordGood.passwordLengthCheck
      }
    ];

    return items;
  }, [password]);

  const allCompleted = useMemo(() => {
    return items.every((item) => item.check);
  }, [items]);

  useEffect(() => {
    if (show && onCheckChange) {
      onCheckChange(allCompleted);
    }
  }, [show, allCompleted, onCheckChange]);

  const PasswordCheck: React.FC<{
    passwordGood: boolean;
    text: string;
  }> = ({ passwordGood, text }) => {
    return (
      <div className="flex items-center space-x-1">
        {passwordGood ? (
          <AppMaterialIcons className="text-green-600" icon={'check_circle'} />
        ) : (
          <AppMaterialIcons className="text-red-600" icon={'close'} />
        )}
        <Text size="sm">{text}</Text>
      </div>
    );
  };

  return (
    <AppPopover
      open={show === false ? false : undefined}
      placement={placement}
      content={
        <div className="flex flex-col p-1.5">
          {items.map((item, index) => (
            <PasswordCheck key={index} passwordGood={item.check} text={item.text} />
          ))}
        </div>
      }>
      <div className="flex w-full cursor-pointer items-center space-x-1">
        {children ? (
          children
        ) : allCompleted ? (
          <AppMaterialIcons icon={'check_circle'} size={12} />
        ) : (
          <AppMaterialIcons icon={'info'} size={12} />
        )}
      </div>
    </AppPopover>
  );
};
