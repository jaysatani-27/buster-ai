import React from 'react';
import { BusterFrameLogoWithText } from '@/assets/svg/BusterFrameLogoWithText';
import Link from 'next/link';
import { BUSTER_HOME_PAGE } from '@/routes/externalRoutes';

import PodiumImage from '@/assets/png/podium.png';
import ArtifactImage from '@/assets/png/artifact.png';
import AdobeImage from '@/assets/png/adobe.png';
import BambooImage from '@/assets/png/bamboo.png';
import CartaImage from '@/assets/png/carta.png';
import GuideCX from '@/assets/png/guidecx.png';
import { Text } from '@/components/text/Text';

export const WelcomeToBuster: React.FC<{
  hasUser: boolean;
}> = () => {
  const allImages = [ArtifactImage, AdobeImage, BambooImage, CartaImage, GuideCX, PodiumImage];

  return (
    <div className="flex h-full w-full flex-col justify-between p-10">
      <div>
        <div className="w-[130px]">
          <Link href={BUSTER_HOME_PAGE}>
            <BusterFrameLogoWithText />
          </Link>
        </div>
        <div className="mt-24">
          <h1
            className="mb-3"
            style={{
              fontSize: 48
            }}>
            Welcome to Buster.
          </h1>
          <div className="text-lg">Stand up a PoC in 30 minutes.</div>
        </div>
      </div>
      <div>
        {/* <div>
          <div className="mb-6">Join startups & enterprises</div>
          <div className="flex max-w-[450px] flex-wrap gap-6">
            {allImages.map((image, index) => {
              return (
                <img
                  key={index}
                  src={image.src}
                  alt={`Company logo ${index}`}
                  className="max-h-[24px] object-contain"
                />
              );
            })}
          </div>
        </div> */}
        {/* <Divider className="mt-12 border-black" /> */}

        <div className="flex space-x-8">
          <a href="#">
            <Text>Terms of Service</Text>
          </a>
          <a href="#">
            <Text>Privacy Policy</Text>
          </a>
        </div>
      </div>
    </div>
  );
};
